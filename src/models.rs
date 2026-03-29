use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub repo: String,
    pub state: String,
}

pub struct MemberStats {
    pub username: String,
    pub prs_merged: u32,
    pub prs_open: u32,
    pub total_additions: i64,
    pub total_deletions: i64,
    pub reviews_given: u32,
    pub quality_reviews: u32,
    pub prs_with_tests: u32,
    pub prs_with_docs: u32,
    pub small_prs: u32,
    pub large_prs: u32,
    pub total_commits: u32,
    pub commits_by_repo: HashMap<String, u32>,
    pub score: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReviewedPr {
    pub repo: String,
    pub number: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReviewInfo {
    pub body: Option<String>,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct PrDetails {
    pub files: Vec<PrFile>,
    pub additions: i64,
    pub deletions: i64,
}

#[derive(Debug, Deserialize)]
pub struct PrFile {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct ProcessedPr {
    pub title: String,
    pub url: String,
    pub repo: String,
    pub state: String,
    pub additions: i64,
    pub deletions: i64,
    pub total_changes: i64,
    pub risk: String,
    pub action: String,
    pub test_files: Vec<String>,
    pub doc_files: Vec<String>,
}

impl ProcessedPr {
    pub fn to_markdown(&self) -> String {
        let tests_text = if self.test_files.is_empty() {
            "No explicit test files detected.".to_string()
        } else {
            let files: String = self.test_files.iter().take(3).cloned().collect::<Vec<_>>().join(" ");
            format!("Verified. (Found: {}...)", files)
        };

        let docs_text = if self.doc_files.is_empty() {
            "No documentation files detected.".to_string()
        } else {
            let files: String = self.doc_files.iter().take(3).cloned().collect::<Vec<_>>().join(" ");
            format!("Verified. (Found: {}...)", files)
        };

        format!(
            "- **Scope:** {}\n- **Impact:** {} lines changed (+{} / -{}).\n- **Risk:** {}\n- **Action:** {}\n- **Tests:** {}\n- **Docs:** {}\n  ([View PR]({}))\n",
            self.title,
            self.total_changes,
            self.additions,
            self.deletions,
            self.risk,
            self.action,
            tests_text,
            docs_text,
            self.url
        )
    }
}
