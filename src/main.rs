mod ai;
mod cli;
mod config;
mod github;
mod models;
mod notes;
mod processor;
mod team;

use anyhow::{anyhow, Result};
use chrono::{Duration, Local, Utc};
use clap::Parser;
use std::fs;
use std::path::Path;

use crate::ai::{check_ai_available, concatenate_reports, generate_final_document, generate_notes_summary, generate_team_document, translate_report};
use crate::cli::{Cli, Commands};
use crate::github::{check_gh_auth, check_gh_installed, fetch_commit_counts, fetch_org_members, fetch_prs, fetch_quality_reviews, get_current_user};
use crate::notes::collect_notes;
use crate::processor::{generate_repo_report, process_all_prs};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Generate {
            start_date,
            end_date,
            org,
            repo,
            language,
            model,
            notes,
            cwd,
            team,
            members,
            setup,
        }) => {
            if setup && !team {
                return Err(anyhow!("--setup requires --team"));
            }
            if team {
                if members.is_none() && org.is_none() {
                    return Err(anyhow!("--team requires either --members or --org"));
                }
                if setup {
                    run_team_setup(members, org)?;
                } else {
                    run_team_generate(members, start_date, end_date, org, repo, language, model)?;
                }
            } else {
                run_generate(start_date, end_date, org, repo, language, model, notes, cwd)?;
            }
        }
        None => {
            println!("Usage: promoteme <command> [OPTIONS]");
            println!();
            println!("Commands:");
            println!("  generate    Generate brag document from GitHub contributions");
            println!();
            println!("Run 'promoteme --help' for more information.");
        }
    }

    Ok(())
}

fn run_generate(
    start_date: Option<String>,
    end_date: Option<String>,
    org_filter: Option<String>,
    repo_filter: Option<String>,
    language: Option<String>,
    model: String,
    notes_dir: Option<String>,
    cwd: Option<String>,
) -> Result<()> {
    check_gh_installed()?;
    check_gh_auth()?;

    let current_user = get_current_user()?;
    println!("Fetching PRs for user: {}...", current_user);

    let (start, end) = resolve_dates(start_date, end_date);

    let date_filter = build_date_filter(&start, &end);

    let dir_suffix = build_dir_suffix(&start, &end);

    let output_dir_name = match cwd {
        Some(dir) => dir,
        None => format!("artifacts/{}_{}", current_user, get_timestamp_suffix()),
    };
    let output_dir = Path::new(&output_dir_name);
    fs::create_dir_all(output_dir)?;
    println!("Output directory created: {}", output_dir.display());

    let prs = fetch_prs(
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

    let processed_prs = process_all_prs(&prs);

    if processed_prs.is_empty() {
        println!("No PRs could be processed.");
        return Ok(());
    }

    let mut repo_processed: std::collections::HashMap<String, Vec<&models::ProcessedPr>> =
        std::collections::HashMap::new();
    for pr in &processed_prs {
        repo_processed.entry(pr.repo.clone()).or_default().push(pr);
    }

    let mut reports: Vec<(String, String)> = Vec::new();
    for (repo, prs) in &repo_processed {
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
        // Concatenate all repo content
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

fn resolve_members(members_opt: Option<String>, org_filter: Option<String>) -> Result<Vec<String>> {
    if let Some(members_str) = members_opt {
        let list: Vec<String> = members_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if list.is_empty() {
            return Err(anyhow!("No valid members provided in --members"));
        }
        return Ok(list);
    }

    let orgs_str = org_filter
        .ok_or_else(|| anyhow!("You must specify either --members or --org"))?;
    let orgs: Vec<&str> = orgs_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut all_members: Vec<String> = Vec::new();
    for org in &orgs {
        println!("Fetching members from org {}...", org);
        let org_members = fetch_org_members(org)?;
        println!("Found {} members in {}.", org_members.len(), org);
        for m in org_members {
            if seen.insert(m.clone()) {
                all_members.push(m);
            }
        }
    }

    if all_members.is_empty() {
        return Err(anyhow!("No members found in the specified org(s)"));
    }
    Ok(all_members)
}

fn run_team_setup(members_opt: Option<String>, org_filter: Option<String>) -> Result<()> {
    check_gh_installed()?;
    check_gh_auth()?;

    let team_name = org_filter.as_deref().unwrap_or("team").to_string();
    let members = resolve_members(members_opt, org_filter)?;
    let path = config::generate_setup_file(&members, &team_name)?;
    println!("Created {} with {} members (all defaulting to entrylevel).", path.display(), members.len());
    println!("Edit {}/team.json to set levels, then run: promoteme generate --team --org ...", team_name);
    println!("Valid levels: entrylevel, mid, senior, tech_lead, specialist, architect, manager");
    Ok(())
}

fn run_team_generate(
    members_opt: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    org_filter: Option<String>,
    repo_filter: Option<String>,
    language: Option<String>,
    model: String,
) -> Result<()> {
    check_gh_installed()?;
    check_gh_auth()?;

    let team_name = org_filter.as_deref().unwrap_or("team").to_string();
    let members = resolve_members(members_opt, org_filter.clone())?;

    let config_dir = Path::new(&team_name);
    let team_config = config::load_team_config(config_dir)?;
    if team_config.is_some() {
        println!("Loaded team config from {}.", config_dir.join("team.json").display());
    }

    println!("Team mode: analyzing {} members...", members.len());

    let (start, end) = resolve_dates(start_date, end_date);
    let date_filter = build_date_filter(&start, &end);

    let timestamp = Local::now().format("%Y_%m_%d_%H_%M").to_string();
    let output_dir_name = format!("{}/{}", team_name, timestamp);
    let output_dir = Path::new(&output_dir_name);
    fs::create_dir_all(output_dir)?;
    println!("Output directory created: {}", output_dir.display());

    let mut all_stats: Vec<models::MemberStats> = Vec::new();
    let mut members_content = String::new();

    for member in &members {
        println!("Fetching PRs for {}...", member);

        let prs = fetch_prs(
            member,
            date_filter.as_deref(),
            org_filter.as_deref(),
            repo_filter.as_deref(),
        )?;

        if prs.is_empty() {
            println!("No contributions found for {}.", member);
            continue;
        }

        println!("Found {} PRs for {}. Processing...", prs.len(), member);

        let processed_prs = process_all_prs(&prs);

        let (total_reviews, quality_reviews) = match fetch_quality_reviews(
            member,
            date_filter.as_deref(),
            org_filter.as_deref(),
            repo_filter.as_deref(),
        ) {
            Ok(counts) => counts,
            Err(e) => {
                eprintln!("Warning: Could not fetch reviews for {}: {}", member, e);
                (0, 0)
            }
        };

        let commits_by_repo = match fetch_commit_counts(
            member,
            start.as_deref(),
            end.as_deref(),
            org_filter.as_deref(),
            repo_filter.as_deref(),
        ) {
            Ok(counts) => counts,
            Err(e) => {
                eprintln!("Warning: Could not fetch commits for {}: {}", member, e);
                std::collections::HashMap::new()
            }
        };

        let stats = team::compute_member_stats(member, &processed_prs, total_reviews, quality_reviews, commits_by_repo);
        let report = team::generate_member_report(&stats, &processed_prs);

        let member_path = output_dir.join(format!("{}.md", member));
        fs::write(&member_path, &report)?;
        println!("Saved report: {}", member_path.display());

        members_content.push_str("\n\n---\n");
        members_content.push_str(&format!("Member: {}\n", member));
        if let Some(ref cfg) = team_config {
            if let Some(mc) = cfg.members.get(member) {
                members_content.push_str(&format!("Level: {}", mc.level.as_str()));
                if let Some(ref role) = mc.role {
                    members_content.push_str(&format!(" | Role: {}", role));
                }
                members_content.push('\n');
            }
        }
        members_content.push_str(&report);

        all_stats.push(stats);
    }

    if all_stats.is_empty() {
        println!("No contributions found for any team member.");
        return Ok(());
    }

    let scores_table = team::generate_scores_table(&all_stats);
    let scores_path = output_dir.join("SCORES.md");
    fs::write(&scores_path, &scores_table)?;
    println!("Saved scores: {}", scores_path.display());

    members_content.push_str("\n\n---\n");
    members_content.push_str(&scores_table);

    let readme_path = output_dir.join("README.md");
    if check_ai_available(&model) {
        println!("Generating team document...");
        let team_doc = generate_team_document(&model, &members_content, language.as_deref())?;
        fs::write(&readme_path, &team_doc)?;
        println!("Team document generated using {}: {}", model, readme_path.display());
    } else {
        println!("'{}' CLI not found. Writing raw content instead.", model);
        fs::write(&readme_path, &members_content)?;
        println!("Team document written: {}", readme_path.display());
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

fn get_timestamp_suffix() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}
