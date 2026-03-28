use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::github::fetch_pr_details;
use crate::models::{ProcessedPr, SearchResult};

fn is_doc_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    let in_docs_dir = lower.starts_with("docs/") || lower.contains("/docs/");
    let doc_extension = lower.ends_with(".md") || lower.ends_with(".mdx") || lower.ends_with(".rst");
    let basename = path.rsplit('/').next().unwrap_or(path).to_uppercase();
    let doc_filename = basename.starts_with("README")
        || basename.starts_with("CHANGELOG")
        || basename.starts_with("CONTRIBUTING");
    in_docs_dir || doc_extension || doc_filename
}

/// Process a single PR to extract details
pub fn process_pr(pr: &SearchResult) -> Result<ProcessedPr> {
    let details = fetch_pr_details(&pr.url)?;

    let total_changes = details.additions + details.deletions;

    let risk = if total_changes > 500 {
        "High (Large changeset)".to_string()
    } else if total_changes > 200 {
        "Medium (Moderate changeset)".to_string()
    } else {
        "Low (Small changeset)".to_string()
    };

    let action = match pr.state.to_uppercase().as_str() {
        "OPEN" => "Needs Review".to_string(),
        "MERGED" => "No action (Merged)".to_string(),
        _ => "No action (Closed)".to_string(),
    };

    let test_files: Vec<String> = details
        .files
        .iter()
        .filter(|f| f.path.contains("test") || f.path.contains("spec"))
        .map(|f| f.path.clone())
        .collect();

    let doc_files: Vec<String> = details
        .files
        .iter()
        .filter(|f| is_doc_file(&f.path))
        .map(|f| f.path.clone())
        .collect();

    Ok(ProcessedPr {
        title: pr.title.clone(),
        url: pr.url.clone(),
        repo: pr.repo.clone(),
        state: pr.state.clone(),
        additions: details.additions,
        deletions: details.deletions,
        total_changes,
        risk,
        action,
        test_files,
        doc_files,
    })
}

/// Process all PRs in parallel with progress bar
pub fn process_all_prs(prs: &[SearchResult]) -> Vec<ProcessedPr> {
    let pb = ProgressBar::new(prs.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} PRs")
            .unwrap()
            .progress_chars("#>-"),
    );

    let results: Vec<ProcessedPr> = prs
        .par_iter()
        .filter_map(|pr| {
            let result = process_pr(pr);
            pb.inc(1);
            match result {
                Ok(processed) => Some(processed),
                Err(e) => {
                    eprintln!("Warning: Failed to fetch details for '{}': {}", pr.title, e);
                    None
                }
            }
        })
        .collect();

    pb.finish_with_message("Done processing PRs");
    results
}

/// Generate markdown report for a repository
pub fn generate_repo_report(repo: &str, prs: &[ProcessedPr]) -> String {
    let mut report = format!("# Report for {}\n\n", repo);

    for pr in prs {
        report.push_str(&pr.to_markdown());
        report.push('\n');
    }

    report
}
