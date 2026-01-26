use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::process::Command;

use crate::models::{PrDetails, PrFile, SearchResult};
use crate::provider::PrProvider;

pub struct AzureDevOpsProvider {
    organization: Option<String>,
}

impl AzureDevOpsProvider {
    pub fn new(organization: Option<String>) -> Self {
        Self { organization }
    }
}

#[derive(Debug, Deserialize)]
struct AzurePr {
    #[serde(rename = "pullRequestId")]
    pull_request_id: i64,
    title: String,
    #[serde(rename = "creationDate")]
    creation_date: String,
    status: String,
    repository: AzureRepository,
}

#[derive(Debug, Deserialize)]
struct AzureRepository {
    name: String,
    project: AzureProject,
    #[serde(rename = "webUrl")]
    web_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AzureProject {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ProfileResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct AccountsResponse {
    value: Vec<AccountInfo>,
}

#[derive(Debug, Deserialize)]
struct AccountInfo {
    #[serde(rename = "accountName")]
    account_name: String,
}

#[derive(Debug, Deserialize)]
struct ProjectListResponse {
    value: Vec<ProjectInfo>,
}

#[derive(Debug, Deserialize)]
struct ProjectInfo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct AzurePrDetails {
    description: Option<String>,
}

impl PrProvider for AzureDevOpsProvider {
    fn check_installed(&self) -> Result<()> {
        let output = Command::new("which").arg("az").output()?;

        if !output.status.success() {
            bail!("Azure CLI ('az') is not installed. Install it from https://docs.microsoft.com/en-us/cli/azure/install-azure-cli");
        }
        Ok(())
    }

    fn check_auth(&self) -> Result<()> {
        let output = Command::new("az")
            .args(["account", "show"])
            .output()?;

        if !output.status.success() {
            bail!("You are not logged into Azure CLI. Run 'az login' first.");
        }

        let devops_output = Command::new("az")
            .args(["devops", "project", "list"])
            .output();

        if let Ok(out) = devops_output {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if stderr.contains("az devops login") || stderr.contains("credential") {
                    bail!("Azure DevOps requires authentication. Run 'az devops login' or set a PAT token.");
                }
            }
        }

        Ok(())
    }

    fn get_current_user(&self) -> Result<String> {
        let output = Command::new("az")
            .args(["ad", "signed-in-user", "show", "--query", "mail", "-o", "tsv"])
            .output()
            .context("Failed to get current user")?;

        if !output.status.success() {
            let output = Command::new("az")
                .args(["account", "show", "--query", "user.name", "-o", "tsv"])
                .output()
                .context("Failed to get current user from account")?;

            if !output.status.success() {
                bail!("Could not retrieve current Azure DevOps user.");
            }

            let user = String::from_utf8(output.stdout)?
                .trim()
                .to_string();

            if user.is_empty() {
                bail!("Could not retrieve current Azure DevOps user.");
            }

            return Ok(user);
        }

        let user = String::from_utf8(output.stdout)?
            .trim()
            .to_string();

        if user.is_empty() {
            let output = Command::new("az")
                .args(["account", "show", "--query", "user.name", "-o", "tsv"])
                .output()
                .context("Failed to get current user from account")?;

            let user = String::from_utf8(output.stdout)?
                .trim()
                .to_string();

            if user.is_empty() {
                bail!("Could not retrieve current Azure DevOps user.");
            }

            return Ok(user);
        }

        Ok(user)
    }

    fn fetch_prs(
        &self,
        user: &str,
        _date_filter: Option<&str>,
        org_filter: Option<&str>,
        repo_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let orgs = if let Some(org) = org_filter.or(self.organization.as_deref()) {
            org.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            eprintln!("No organization specified, discovering organizations...");
            let token = get_access_token()?;
            let member_id = get_member_id(&token)?;
            let discovered = discover_organizations(&token, &member_id)?;
            if discovered.is_empty() {
                bail!("No Azure DevOps organizations found for your account. Specify one with --org.");
            }
            eprintln!("Found {} organization(s): {}", discovered.len(), discovered.join(", "));
            discovered
        };

        let mut all_results = Vec::new();

        for org in &orgs {
            let projects = if let Some(repo) = repo_filter {
                if repo.contains('/') {
                    let parts: Vec<&str> = repo.split('/').collect();
                    vec![parts[0].to_string()]
                } else {
                    discover_projects(org)?
                }
            } else {
                eprintln!("Discovering projects in organization '{}'...", org);
                let discovered = discover_projects(org)?;
                if !discovered.is_empty() {
                    eprintln!("Found {} project(s) in '{}': {}", discovered.len(), org, discovered.join(", "));
                }
                discovered
            };

            for project in &projects {
                let repo_name = repo_filter.and_then(|r| {
                    if r.contains('/') {
                        r.split('/').nth(1).map(|s| s.to_string())
                    } else {
                        Some(r.to_string())
                    }
                });

                match fetch_prs_for_org_project(user, org, project, repo_name.as_deref()) {
                    Ok(prs) => all_results.extend(prs),
                    Err(e) => eprintln!("Warning: Failed to fetch PRs from {}/{}: {}", org, project, e),
                }
            }
        }

        Ok(all_results)
    }

    fn fetch_pr_details(&self, url: &str) -> Result<PrDetails> {
        let pr_id = extract_pr_id_from_url(url)?;
        let org_from_url = extract_org_from_url(url);

        let mut args = vec![
            "repos".to_string(),
            "pr".to_string(),
            "show".to_string(),
            "--id".to_string(),
            pr_id.to_string(),
            "-o".to_string(),
            "json".to_string(),
        ];

        let org = org_from_url.or_else(|| self.organization.clone());
        if let Some(ref org_name) = org {
            let org_url = if org_name.starts_with("https://") {
                org_name.clone()
            } else {
                format!("https://dev.azure.com/{}", org_name)
            };
            args.push("--org".to_string());
            args.push(org_url);
        }

        let output = Command::new("az")
            .args(&args)
            .output()
            .context("Failed to fetch PR details from Azure DevOps")?;

        if !output.status.success() {
            bail!("Failed to fetch details for PR: {}", url);
        }

        let stdout = String::from_utf8(output.stdout)?;
        let azure_details: AzurePrDetails = serde_json::from_str(&stdout)
            .context("Failed to parse Azure DevOps PR details")?;

        let files = fetch_pr_files(&org, pr_id);

        let (additions, deletions) = estimate_changes_from_files(&files);

        Ok(PrDetails {
            body: azure_details.description,
            files,
            additions,
            deletions,
        })
    }
}

fn extract_pr_id_from_url(url: &str) -> Result<i64> {
    if let Some(id_str) = url.strip_prefix("PR #") {
        return id_str.parse().context("Invalid PR ID");
    }

    if url.contains("/pullrequest/") {
        let parts: Vec<&str> = url.split("/pullrequest/").collect();
        if parts.len() == 2 {
            let id_part = parts[1].split('?').next().unwrap_or(parts[1]);
            return id_part.parse().context("Invalid PR ID in URL");
        }
    }

    if url.contains("/_git/") && url.contains("/pullrequest/") {
        let parts: Vec<&str> = url.rsplitn(2, "/pullrequest/").collect();
        if !parts.is_empty() {
            let id_part = parts[0].split('?').next().unwrap_or(parts[0]);
            return id_part.parse().context("Invalid PR ID in Azure DevOps URL");
        }
    }

    bail!("Could not extract PR ID from URL: {}", url);
}

fn extract_org_from_url(url: &str) -> Option<String> {
    if url.starts_with("https://dev.azure.com/") {
        let path = url.strip_prefix("https://dev.azure.com/")?;
        let org = path.split('/').next()?;
        if !org.is_empty() {
            return Some(org.to_string());
        }
    }

    if url.contains(".visualstudio.com/") {
        let parts: Vec<&str> = url.split(".visualstudio.com").collect();
        if !parts.is_empty() {
            let org = parts[0]
                .strip_prefix("https://")?
                .trim_end_matches('/');
            if !org.is_empty() {
                return Some(org.to_string());
            }
        }
    }

    None
}

fn fetch_pr_files(_organization: &Option<String>, _pr_id: i64) -> Vec<PrFile> {
    Vec::new()
}

fn estimate_changes_from_files(files: &[PrFile]) -> (i64, i64) {
    let file_count = files.len() as i64;
    let estimated = file_count * 50;
    (estimated, estimated / 4)
}

fn get_access_token() -> Result<String> {
    let output = Command::new("az")
        .args([
            "account",
            "get-access-token",
            "--resource",
            "499b84ac-1321-427f-aa17-267ca6975798",
            "--query",
            "accessToken",
            "-o",
            "tsv",
        ])
        .output()
        .context("Failed to get access token from Azure CLI")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get Azure DevOps access token: {}", stderr);
    }

    let token = String::from_utf8(output.stdout)?
        .trim()
        .to_string();

    if token.is_empty() {
        bail!("Received empty access token from Azure CLI");
    }

    Ok(token)
}

fn get_member_id(token: &str) -> Result<String> {
    let response: ProfileResponse = ureq::get("https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=7.1")
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .context("Failed to fetch profile from Azure DevOps")?
        .into_json()
        .context("Failed to parse profile response")?;

    Ok(response.id)
}

fn discover_organizations(token: &str, member_id: &str) -> Result<Vec<String>> {
    let url = format!(
        "https://app.vssps.visualstudio.com/_apis/accounts?memberId={}&api-version=7.1",
        member_id
    );

    let response: AccountsResponse = ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .context("Failed to fetch organizations from Azure DevOps")?
        .into_json()
        .context("Failed to parse organizations response")?;

    Ok(response.value.into_iter().map(|a| a.account_name).collect())
}

fn discover_projects(org: &str) -> Result<Vec<String>> {
    let org_url = if org.starts_with("https://") {
        org.to_string()
    } else {
        format!("https://dev.azure.com/{}", org)
    };

    let output = Command::new("az")
        .args([
            "devops",
            "project",
            "list",
            "--org",
            &org_url,
            "-o",
            "json",
        ])
        .output()
        .context("Failed to list projects from Azure DevOps")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to list projects for org '{}': {}", org, stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let projects: ProjectListResponse = serde_json::from_str(&stdout)
        .context("Failed to parse project list response")?;

    Ok(projects.value.into_iter().map(|p| p.name).collect())
}

fn fetch_prs_for_org_project(
    user: &str,
    org: &str,
    project: &str,
    repo: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let org_name = if org.starts_with("https://") {
        org.trim_start_matches("https://dev.azure.com/")
            .trim_end_matches('/')
            .to_string()
    } else {
        org.to_string()
    };

    let org_url = format!("https://dev.azure.com/{}", org_name);

    let mut args = vec![
        "repos",
        "pr",
        "list",
        "--creator",
        user,
        "--status",
        "all",
        "--org",
        &org_url,
        "--project",
        project,
        "-o",
        "json",
    ];

    let repo_owned;
    if let Some(r) = repo {
        repo_owned = r.to_string();
        args.push("--repository");
        args.push(&repo_owned);
    }

    let output = Command::new("az")
        .args(&args)
        .output()
        .context("Failed to fetch PRs from Azure DevOps")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to fetch PRs: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let azure_prs: Vec<AzurePr> = serde_json::from_str(&stdout)
        .context("Failed to parse Azure DevOps PR list")?;

    let results: Vec<SearchResult> = azure_prs
        .into_iter()
        .map(|pr| {
            let repo_name = format!("{}/{}", pr.repository.project.name, pr.repository.name);
            let url = pr.repository.web_url
                .map(|base| format!("{}/pullrequest/{}", base, pr.pull_request_id))
                .unwrap_or_else(|| {
                    format!(
                        "https://dev.azure.com/{}/_git/{}/pullrequest/{}",
                        org_name, pr.repository.name, pr.pull_request_id
                    )
                });

            let state = match pr.status.as_str() {
                "completed" => "closed",
                "active" => "open",
                "abandoned" => "closed",
                _ => &pr.status,
            };

            SearchResult {
                title: pr.title,
                url,
                repo: repo_name,
                created_at: pr.creation_date,
                state: state.to_string(),
            }
        })
        .collect();

    Ok(results)
}
