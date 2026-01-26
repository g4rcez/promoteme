use anyhow::Result;

use crate::models::{PrDetails, SearchResult};

pub trait PrProvider {
    fn check_installed(&self) -> Result<()>;
    fn check_auth(&self) -> Result<()>;
    fn get_current_user(&self) -> Result<String>;
    fn fetch_prs(
        &self,
        user: &str,
        date_filter: Option<&str>,
        org_filter: Option<&str>,
        repo_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;
    fn fetch_pr_details(&self, url: &str) -> Result<PrDetails>;
}
