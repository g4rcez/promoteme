use std::collections::HashMap;

use crate::models::{MemberStats, ProcessedPr};

const SCORE_PR_MERGED: i64 = 10;
const SCORE_PR_OPEN: i64 = 3;
const SCORE_QUALITY_REVIEW: i64 = 8;
const SCORE_PR_WITH_TESTS: i64 = 3;
const SCORE_PR_WITH_DOCS: i64 = 2;
const SCORE_SMALL_PR: i64 = 2;
const SCORE_LARGE_PR_PENALTY: i64 = 1;

pub fn compute_member_stats(
    username: &str,
    processed_prs: &[ProcessedPr],
    total_reviews: usize,
    quality_reviews: usize,
    commits_by_repo: HashMap<String, u32>,
) -> MemberStats {
    let mut prs_merged = 0u32;
    let mut prs_open = 0u32;
    let mut total_additions = 0i64;
    let mut total_deletions = 0i64;
    let mut prs_with_tests = 0u32;
    let mut prs_with_docs = 0u32;
    let mut small_prs = 0u32;
    let mut large_prs = 0u32;

    for pr in processed_prs {
        match pr.state.to_uppercase().as_str() {
            "MERGED" => prs_merged += 1,
            "OPEN" => prs_open += 1,
            _ => {}
        }
        total_additions += pr.additions;
        total_deletions += pr.deletions;
        if !pr.test_files.is_empty() {
            prs_with_tests += 1;
        }
        if !pr.doc_files.is_empty() {
            prs_with_docs += 1;
        }
        if pr.total_changes <= 200 {
            small_prs += 1;
        }
        if pr.total_changes > 500 {
            large_prs += 1;
        }
    }

    let reviews_given = total_reviews as u32;
    let quality_reviews_u32 = quality_reviews as u32;
    let total_commits: u32 = commits_by_repo.values().sum();
    let score = prs_merged as i64 * SCORE_PR_MERGED
        + prs_open as i64 * SCORE_PR_OPEN
        + quality_reviews_u32 as i64 * SCORE_QUALITY_REVIEW
        + prs_with_tests as i64 * SCORE_PR_WITH_TESTS
        + prs_with_docs as i64 * SCORE_PR_WITH_DOCS
        + small_prs as i64 * SCORE_SMALL_PR
        - large_prs as i64 * SCORE_LARGE_PR_PENALTY;

    MemberStats {
        username: username.to_string(),
        prs_merged,
        prs_open,
        total_additions,
        total_deletions,
        reviews_given,
        quality_reviews: quality_reviews_u32,
        prs_with_tests,
        prs_with_docs,
        small_prs,
        large_prs,
        total_commits,
        commits_by_repo,
        score,
    }
}

pub fn generate_member_report(stats: &MemberStats, prs: &[ProcessedPr]) -> String {
    let mut report = format!("# Contributions: {}\n\n", stats.username);

    report.push_str("## Stats\n\n");
    report.push_str(&format!("- **PRs Merged:** {}\n", stats.prs_merged));
    report.push_str(&format!("- **PRs Open:** {}\n", stats.prs_open));
    report.push_str(&format!("- **Reviews Given:** {}\n", stats.reviews_given));
    report.push_str(&format!("- **Quality Reviews:** {}\n", stats.quality_reviews));
    report.push_str(&format!("- **PRs with Tests:** {}\n", stats.prs_with_tests));
    report.push_str(&format!("- **PRs with Docs:** {}\n", stats.prs_with_docs));
    report.push_str(&format!("- **Small PRs (<= 200 lines):** {}\n", stats.small_prs));
    report.push_str(&format!("- **Large PRs (> 500 lines):** {}\n", stats.large_prs));
    report.push_str(&format!("- **Total Commits:** {}\n", stats.total_commits));
    report.push_str(&format!("- **Score:** {}\n", stats.score));
    report.push('\n');

    let mut by_repo: HashMap<&str, Vec<&ProcessedPr>> = HashMap::new();
    for pr in prs {
        by_repo.entry(pr.repo.as_str()).or_default().push(pr);
    }

    let mut repos: Vec<&&str> = by_repo.keys().collect();
    repos.sort();

    for repo in repos {
        let commits = stats.commits_by_repo.get(*repo).copied().unwrap_or(0);
        report.push_str(&format!("## {} ({} commits)\n\n", repo, commits));
        for pr in &by_repo[repo] {
            report.push_str(&pr.to_markdown());
            report.push('\n');
        }
    }

    report
}

pub fn generate_scores_table(all_stats: &[MemberStats]) -> String {
    let mut sorted: Vec<&MemberStats> = all_stats.iter().collect();
    sorted.sort_by(|a, b| b.score.cmp(&a.score));

    let mut table = "# Team Scores\n\n".to_string();
    table.push_str("| Rank | Member | Score | Merged | Reviews (Quality) | With Tests | With Docs | Commits | Avg Size |\n");
    table.push_str("|------|--------|-------|--------|-------------------|------------|-----------|---------|----------|\n");

    for (i, stats) in sorted.iter().enumerate() {
        let total_prs = stats.prs_merged + stats.prs_open;
        let avg_size = if total_prs > 0 {
            (stats.total_additions + stats.total_deletions) / total_prs as i64
        } else {
            0
        };
        table.push_str(&format!(
            "| {} | {} | {} | {} | {} ({}) | {} | {} | {} | {} |\n",
            i + 1,
            stats.username,
            stats.score,
            stats.prs_merged,
            stats.reviews_given,
            stats.quality_reviews,
            stats.prs_with_tests,
            stats.prs_with_docs,
            stats.total_commits,
            avg_size,
        ));
    }

    table
}
