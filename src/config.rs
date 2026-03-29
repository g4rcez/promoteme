use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MemberLevel {
    #[serde(rename = "entrylevel")]
    EntryLevel,
    Mid,
    Senior,
    TechLead,
    Specialist,
    Architect,
    Manager,
}

impl MemberLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberLevel::EntryLevel => "entrylevel",
            MemberLevel::Mid => "mid",
            MemberLevel::Senior => "senior",
            MemberLevel::TechLead => "tech_lead",
            MemberLevel::Specialist => "specialist",
            MemberLevel::Architect => "architect",
            MemberLevel::Manager => "manager",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemberConfig {
    pub level: MemberLevel,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamConfig {
    pub members: HashMap<String, MemberConfig>,
}

pub fn generate_setup_file(members: &[String], dir: &str) -> Result<PathBuf> {
    let mut map = HashMap::new();
    for member in members {
        map.insert(
            member.clone(),
            MemberConfig {
                level: MemberLevel::EntryLevel,
                role: None,
            },
        );
    }
    let config = TeamConfig { members: map };
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::create_dir_all(dir)?;
    let path = PathBuf::from(format!("{}/team.json", dir));
    std::fs::write(&path, json)?;
    Ok(path)
}

pub fn load_team_config(dir: &Path) -> Result<Option<TeamConfig>> {
    let path = dir.join("team.json");
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let config: TeamConfig = serde_json::from_str(&content)?;
    Ok(Some(config))
}
