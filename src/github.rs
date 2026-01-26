use anyhow::{bail, Context, Result};
use std::process::Command;

use crate::models::{PrDetails, SearchResult};
use crate::provider::PrProvider;

pub struct GitHubProvider;

impl GitHubProvider {
    pub fn new() -> Self {
        Self
    }
}

impl PrProvider for GitHubProvider {
    fn check_installed(&self) -> Result<()> {
        let output = Command::new("which").arg("gh").output()?;

        if !output.status.success() {
            bail!("GitHub CLI ('gh') is not installed.");
        }
        Ok(())
    }

    fn check_auth(&self) -> Result<()> {
        let status = Command::new("gh")
            .args(["auth", "status"])
            .output()?
            .status;

        if !status.success() {
            bail!("You are not logged into GitHub CLI. Run 'gh auth login' first.");
        }
        Ok(())
    }

    fn get_current_user(&self) -> Result<String> {
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

    fn fetch_prs(
        &self,
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
                r#".items[] | {title: .title, url: .html_url, repo: (.repository_url | sub("https://api.github.com/repos/"; "")), created_at: .created_at, state: .state}"#,
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

    fn fetch_pr_details(&self, url: &str) -> Result<PrDetails> {
        let output = Command::new("gh")
            .args(["pr", "view", url, "--json", "body,files,additions,deletions"])
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
}
