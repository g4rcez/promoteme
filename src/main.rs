mod ai;
mod azuredevops;
mod cli;
mod github;
mod models;
mod notes;
mod processor;
mod provider;

use anyhow::Result;
use chrono::{Duration, Utc};
use clap::Parser;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::ai::{check_ai_available, concatenate_reports, generate_final_document, generate_notes_summary, translate_report};
use crate::azuredevops::AzureDevOpsProvider;
use crate::cli::{Cli, Commands, Source};
use crate::github::GitHubProvider;
use crate::notes::collect_notes;
use crate::processor::{generate_repo_report, group_prs_by_repo, process_all_prs};
use crate::provider::PrProvider;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Generate {
            source,
            start_date,
            end_date,
            org,
            repo,
            language,
            model,
            notes,
        }) => {
            run_generate(source, start_date, end_date, org, repo, language, model, notes)?;
        }
        None => {
            println!("Usage: promoteme <command> [OPTIONS]");
            println!();
            println!("Commands:");
            println!("  generate    Generate brag document from GitHub or Azure DevOps contributions");
            println!();
            println!("Run 'promoteme --help' for more information.");
        }
    }

    Ok(())
}

fn create_provider(source: Source, org: Option<&str>) -> Arc<dyn PrProvider + Send + Sync> {
    match source {
        Source::Github => Arc::new(GitHubProvider::new()),
        Source::Azuredevops => Arc::new(AzureDevOpsProvider::new(org.map(String::from))),
    }
}

fn run_generate(
    source: Source,
    start_date: Option<String>,
    end_date: Option<String>,
    org_filter: Option<String>,
    repo_filter: Option<String>,
    language: Option<String>,
    model: String,
    notes_dir: Option<String>,
) -> Result<()> {
    let provider = create_provider(source, org_filter.as_deref());

    provider.check_installed()?;
    provider.check_auth()?;

    let current_user = provider.get_current_user()?;
    println!("Fetching PRs for user: {} from {}...", current_user, source);

    let (start, end) = resolve_dates(start_date, end_date);

    let date_filter = build_date_filter(&start, &end);

    let dir_suffix = build_dir_suffix(&start, &end);

    let output_dir = Path::new(&current_user);
    fs::create_dir_all(output_dir)?;
    println!("Output directory created: {}", output_dir.display());

    let prs = provider.fetch_prs(
        &current_user,
        date_filter.as_deref(),
        org_filter.as_deref(),
        repo_filter.as_deref(),
    )?;

    if prs.is_empty() {
        println!("No contributions found for the specified criteria.");
        return Ok(());
    }

    println!("Found {} PRs. Processing...", prs.len());

    let processed_prs = process_all_prs(&prs, provider.clone());

    if processed_prs.is_empty() {
        println!("No PRs could be processed.");
        return Ok(());
    }

    let grouped = group_prs_by_repo(
        processed_prs
            .iter()
            .map(|p| models::SearchResult {
                title: p.title.clone(),
                url: p.url.clone(),
                repo: p.repo.clone(),
                created_at: p.created_at.clone(),
                state: p.state.clone(),
            })
            .collect(),
    );

    let mut repo_processed: std::collections::HashMap<String, Vec<&models::ProcessedPr>> =
        std::collections::HashMap::new();
    for pr in &processed_prs {
        repo_processed.entry(pr.repo.clone()).or_default().push(pr);
    }

    let mut reports: Vec<(String, String)> = Vec::new();
    for (repo, _) in &grouped {
        if let Some(prs) = repo_processed.get(repo) {
            let prs_owned: Vec<models::ProcessedPr> = prs.iter().map(|p| (*p).clone()).collect();
            let report = generate_repo_report(repo, &prs_owned);

            let final_report = if let Some(ref lang) = language {
                if check_ai_available(&model) {
                    translate_report(&model, &report, lang)?
                } else {
                    report
                }
            } else {
                report
            };

            let filename = repo.replace('/', "_");
            let report_path = output_dir.join(format!("{}.md", filename));
            fs::write(&report_path, &final_report)?;
            println!("Saved report: {}", report_path.display());

            reports.push((repo.clone(), final_report));
        }
    }

    let notes_content = if let Some(ref notes_path) = notes_dir {
        let notes_path = Path::new(notes_path);
        if notes_path.is_dir() {
            let content = collect_notes(notes_path)?;
            if !content.is_empty() {
                if check_ai_available(&model) {
                    println!("Generating notes summary...");
                    let summary = generate_notes_summary(&model, &content, language.as_deref())?;
                    let notes_summary_path = output_dir.join("NOTES_SUMMARY.md");
                    fs::write(&notes_summary_path, &summary)?;
                    println!("Notes summary: {}", notes_summary_path.display());
                }
                Some(content)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let final_doc_path = output_dir.join("README.md");
    println!("Generating final consolidated brag document...");

    if check_ai_available(&model) {
        let mut repo_content = String::new();
        for (repo, report) in &reports {
            repo_content.push_str("\n\n---\n");
            repo_content.push_str(&format!("Content from {}.md:\n", repo.replace('/', "_")));
            repo_content.push_str(report);
        }

        let final_doc = generate_final_document(
            &model,
            &repo_content,
            notes_content.as_deref(),
            language.as_deref(),
        )?;

        fs::write(&final_doc_path, &final_doc)?;
        println!("Final document generated using {}: {}", model, final_doc_path.display());
    } else {
        println!("'{}' CLI not found. Concatenating files instead.", model);
        let final_doc = concatenate_reports(&reports, &dir_suffix);
        fs::write(&final_doc_path, &final_doc)?;
        println!("Final Brag Document concatenated: {}", final_doc_path.display());
    }

    Ok(())
}

fn resolve_dates(start: Option<String>, end: Option<String>) -> (Option<String>, Option<String>) {
    if start.is_none() && end.is_none() {
        let now = Utc::now();
        let six_months_ago = now - Duration::days(180);
        (
            Some(six_months_ago.format("%Y-%m-%d").to_string()),
            Some(now.format("%Y-%m-%d").to_string()),
        )
    } else {
        (start, end)
    }
}

fn build_date_filter(start: &Option<String>, end: &Option<String>) -> Option<String> {
    match (start, end) {
        (Some(s), Some(e)) => Some(format!("created:{}..{}", s, e)),
        (Some(s), None) => Some(format!("created:>={}", s)),
        (None, Some(e)) => Some(format!("created:<={}", e)),
        (None, None) => None,
    }
}

fn build_dir_suffix(start: &Option<String>, end: &Option<String>) -> String {
    match (start, end) {
        (Some(s), Some(e)) => format!("{}_{}", s, e),
        (Some(s), None) => format!("{}_ONWARDS", s),
        (None, Some(e)) => format!("UNTIL_{}", e),
        (None, None) => "ALL_TIME".to_string(),
    }
}
