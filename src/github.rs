use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::process::Command;

use crate::models::{PrDetails, ReviewInfo, ReviewedPr, SearchResult};

/// Check if gh CLI is installed
pub fn check_gh_installed() -> Result<()> {
    let output = Command::new("which").arg("gh").output()?;

    if !output.status.success() {
        bail!("GitHub CLI ('gh') is not installed.");
    }
    Ok(())
}

/// Check if user is authenticated with gh
pub fn check_gh_auth() -> Result<()> {
    let status = Command::new("gh")
        .args(["auth", "status"])
        .output()?
        .status;

    if !status.success() {
        bail!("You are not logged into GitHub CLI. Run 'gh auth login' first.");
    }
    Ok(())
}

/// Get current authenticated GitHub user
pub fn get_current_user() -> Result<String> {
    let output = Command::new("gh")
        .args(["api", "user", "--jq", ".login"])
        .output()
        .context("Failed to get current user")?;

    if !output.status.success() {
        bail!("Could not retrieve current GitHub user.");
    }

    let user = String::from_utf8(output.stdout)?
        .trim()
        .to_string();

    if user.is_empty() {
        bail!("Could not retrieve current GitHub user.");
    }

    Ok(user)
}

/// Fetch PRs for a user with optional filters
pub fn fetch_prs(
    user: &str,
    date_filter: Option<&str>,
    org_filter: Option<&str>,
    repo_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let mut query = format!("author:{} type:pr", user);

    if let Some(date) = date_filter {
        query.push(' ');
        query.push_str(date);
    }

    if let Some(orgs) = org_filter {
        for org in orgs.split(',') {
            query.push_str(&format!(" org:{}", org.trim()));
        }
    }

    if let Some(repos) = repo_filter {
        for repo in repos.split(',') {
            query.push_str(&format!(" repo:{}", repo.trim()));
        }
    }

    let output = Command::new("gh")
        .args([
            "api",
            "-X",
            "GET",
            "search/issues",
            "-f",
            &format!("q={}", query),
            "--paginate",
            "-f",
            "per_page=30",
            "--jq",
            r#".items[] | {title: .title, url: .html_url, repo: (.repository_url | sub("https://api.github.com/repos/"; "")), state: (if .pull_request.merged_at != null then "merged" else .state end)}"#,
        ])
        .output()
        .context("Failed to fetch PRs")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to fetch PRs: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut results = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<SearchResult>(line) {
            Ok(pr) => results.push(pr),
            Err(e) => eprintln!("Warning: Failed to parse PR: {}", e),
        }
    }

    Ok(results)
}

pub fn fetch_reviewed_prs(
    user: &str,
    date_filter: Option<&str>,
    org_filter: Option<&str>,
    repo_filter: Option<&str>,
) -> Result<Vec<ReviewedPr>> {
    let mut query = format!("reviewed-by:{} type:pr", user);

    if let Some(date) = date_filter {
        query.push(' ');
        query.push_str(date);
    }

    if let Some(orgs) = org_filter {
        for org in orgs.split(',') {
            query.push_str(&format!(" org:{}", org.trim()));
        }
    }

    if let Some(repos) = repo_filter {
        for repo in repos.split(',') {
            query.push_str(&format!(" repo:{}", repo.trim()));
        }
    }

    let output = Command::new("gh")
        .args([
            "api",
            "-X",
            "GET",
            "search/issues",
            "-f",
            &format!("q={}", query),
            "--paginate",
            "-f",
            "per_page=100",
            "--jq",
            r#".items[] | {repo: (.repository_url | sub("https://api.github.com/repos/"; "")), number: .number}"#,
        ])
        .output()
        .context("Failed to fetch reviewed PRs")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to fetch reviewed PRs for {}: {}", user, stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut results = Vec::new();
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<ReviewedPr>(line) {
            Ok(rp) => results.push(rp),
            Err(e) => eprintln!("Warning: Failed to parse reviewed PR: {}", e),
        }
    }
    Ok(results)
}

pub fn fetch_pr_reviews(repo: &str, number: u64, user: &str) -> Result<Vec<ReviewInfo>> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/pulls/{}/reviews", repo, number),
            "--jq",
            &format!(
                r#"[.[] | select(.user.login == "{}") | {{body: .body, state: .state}}]"#,
                user
            ),
        ])
        .output()
        .context("Failed to fetch PR reviews")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to fetch reviews for {}/{}: {}", repo, number, stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(serde_json::from_str(stdout.trim()).unwrap_or_default())
}

pub fn is_substantive_review(review: &ReviewInfo) -> bool {
    if review.state.to_uppercase() == "CHANGES_REQUESTED" {
        return true;
    }
    let body = review.body.as_deref().unwrap_or("").trim().to_lowercase();
    if body.is_empty() {
        return false;
    }
    let low_effort = ["lgtm", "approved", "looks good", "looks good to me", "+1", "ship it"];
    !low_effort.iter().any(|p| body == *p)
}

pub fn fetch_quality_reviews(
    user: &str,
    date_filter: Option<&str>,
    org_filter: Option<&str>,
    repo_filter: Option<&str>,
) -> Result<(usize, usize)> {
    let reviewed_prs = fetch_reviewed_prs(user, date_filter, org_filter, repo_filter)?;
    let total = reviewed_prs.len();

    let quality: usize = reviewed_prs
        .par_iter()
        .map(|rp| {
            match fetch_pr_reviews(&rp.repo, rp.number, user) {
                Ok(reviews) => usize::from(reviews.iter().any(is_substantive_review)),
                Err(e) => {
                    eprintln!("Warning: Could not fetch reviews for {}/{}: {}", rp.repo, rp.number, e);
                    1
                }
            }
        })
        .sum();

    Ok((total, quality))
}

/// Fetch all members of a GitHub organization
pub fn fetch_org_members(org: &str) -> Result<Vec<String>> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("orgs/{}/members", org),
            "--paginate",
            "--jq",
            ".[].login",
        ])
        .output()
        .context("Failed to fetch org members")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to fetch members for org {}: {}", org, stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let members: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(members)
}

pub fn fetch_commit_counts(
    user: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
    org_filter: Option<&str>,
    repo_filter: Option<&str>,
) -> Result<HashMap<String, u32>> {
    let date_part = match (start_date, end_date) {
        (Some(s), Some(e)) => format!(" committer-date:{}..{}", s, e),
        (Some(s), None) => format!(" committer-date:>={}", s),
        (None, Some(e)) => format!(" committer-date:<={}", e),
        (None, None) => String::new(),
    };

    let mut query = format!("author:{}{}", user, date_part);

    if let Some(orgs) = org_filter {
        for org in orgs.split(',') {
            query.push_str(&format!(" org:{}", org.trim()));
        }
    }

    if let Some(repos) = repo_filter {
        for repo in repos.split(',') {
            query.push_str(&format!(" repo:{}", repo.trim()));
        }
    }

    let output = Command::new("gh")
        .args([
            "api",
            "-X",
            "GET",
            "search/commits",
            "-H",
            "Accept: application/vnd.github.cloak-preview+json",
            "-f",
            &format!("q={}", query),
            "--paginate",
            "-f",
            "per_page=100",
            "--jq",
            ".items[].repository.full_name",
        ])
        .output()
        .context("Failed to fetch commit counts")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to fetch commits for {}: {}", user, stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut counts: HashMap<String, u32> = HashMap::new();
    for line in stdout.lines() {
        let repo = line.trim();
        if !repo.is_empty() {
            *counts.entry(repo.to_string()).or_insert(0) += 1;
        }
    }

    Ok(counts)
}

/// Fetch detailed PR information
pub fn fetch_pr_details(url: &str) -> Result<PrDetails> {
    let output = Command::new("gh")
        .args(["pr", "view", url, "--json", "files,additions,deletions"])
        .output()
        .context("Failed to fetch PR details")?;

    if !output.status.success() {
        bail!("Failed to fetch details for PR: {}", url);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let details: PrDetails = serde_json::from_str(&stdout)
        .context("Failed to parse PR details")?;

    Ok(details)
}
