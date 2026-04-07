use anyhow::{anyhow, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ai::{check_ai_available, generate_interview_summary, generate_progression_report};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InterviewStep {
    pub number: u32,
    pub title: Option<String>,
    pub date: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InterviewConfig {
    pub company: String,
    pub created_at: String,
    pub steps: Vec<InterviewStep>,
}

impl InterviewConfig {
    pub fn load(company_dir: &Path) -> Result<InterviewConfig> {
        let path = company_dir.join("interview.json");
        let content = fs::read_to_string(&path)?;
        let config: InterviewConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, company_dir: &Path) -> Result<()> {
        let path = company_dir.join("interview.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

fn company_dir(company: &str) -> PathBuf {
    PathBuf::from("interviews").join(company)
}

fn collect_transcripts(dir: &Path) -> Result<String> {
    if !dir.is_dir() {
        return Ok(String::new());
    }

    let mut content = String::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" {
            continue;
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        let file_content = fs::read_to_string(&path)?;

        content.push_str("\n\n---\n");
        content.push_str(&format!("Transcript from {}:\n", filename));
        content.push_str(&file_content);
    }

    Ok(content)
}

pub fn run_interview_init(company: &str) -> Result<()> {
    let dir = company_dir(company);

    if dir.exists() {
        return Err(anyhow!(
            "Directory interviews/{} already exists. Use a different company name or remove the existing directory.",
            company
        ));
    }

    fs::create_dir_all(dir.join("transcripts"))?;
    fs::create_dir_all(dir.join("notes"))?;

    let config = InterviewConfig {
        company: company.to_string(),
        created_at: Local::now().format("%Y-%m-%d").to_string(),
        steps: Vec::new(),
    };
    config.save(&dir)?;

    println!("Initialized interview tracking for '{}'.", company);
    println!("Directory: interviews/{}/", company);
    Ok(())
}

pub fn run_interview_new(
    step: u32,
    company: &str,
    title: Option<String>,
    start_teleprompter: bool,
) -> Result<()> {
    let dir = company_dir(company);

    if !dir.join("interview.json").exists() {
        return Err(anyhow!(
            "Company '{}' not initialized. Run 'promoteme interview init {}' first.",
            company,
            company
        ));
    }

    let mut config = InterviewConfig::load(&dir)?;

    if config.steps.iter().any(|s| s.number == step) {
        return Err(anyhow!("Step {} already exists for company '{}'.", step, company));
    }

    let step_name = format!("step_{:02}", step);
    let transcript_dir = dir.join("transcripts").join(&step_name);
    fs::create_dir_all(&transcript_dir)?;

    let notes_path = dir.join("notes").join(format!("{}.md", step_name));
    let title_suffix = title
        .as_deref()
        .map(|t| format!(" - {}", t))
        .unwrap_or_default();
    let today = Local::now().format("%Y-%m-%d").to_string();
    let notes_template = format!(
        "# Step {}{}\n\nDate: {}\n\n## Notes\n\n",
        step, title_suffix, today
    );
    fs::write(&notes_path, notes_template)?;

    config.steps.push(InterviewStep {
        number: step,
        title,
        date: today,
        status: "active".to_string(),
    });
    config.save(&dir)?;

    println!("Created step {} for company '{}'.", step, company);
    println!("  Transcripts: {}", transcript_dir.display());
    println!("  Notes: {}", notes_path.display());

    if start_teleprompter {
        println!(
            "Teleprompter output path: interviews/{}/transcripts/{}/",
            company, step_name
        );
    }

    Ok(())
}

pub fn run_interview_summarize(
    company: &str,
    step: u32,
    model: String,
    language: Option<String>,
) -> Result<()> {
    let dir = company_dir(company);

    if !dir.join("interview.json").exists() {
        return Err(anyhow!(
            "Company '{}' not initialized. Run 'promoteme interview init {}' first.",
            company,
            company
        ));
    }

    let mut config = InterviewConfig::load(&dir)?;

    if !config.steps.iter().any(|s| s.number == step) {
        let valid: Vec<String> = config.steps.iter().map(|s| s.number.to_string()).collect();
        return Err(anyhow!(
            "Step {} not found for company '{}'. Valid steps: {}",
            step,
            company,
            valid.join(", ")
        ));
    }

    if !check_ai_available(&model) {
        return Err(anyhow!(
            "AI CLI '{}' not found. Install it or specify a different model with -m.",
            model
        ));
    }

    let step_name = format!("step_{:02}", step);

    let transcript_dir = dir.join("transcripts").join(&step_name);
    let transcript_content = collect_transcripts(&transcript_dir)?;
    if transcript_content.is_empty() {
        println!("Warning: no transcript files found in {}", transcript_dir.display());
    }

    let notes_path = dir.join("notes").join(format!("{}.md", step_name));
    let notes_content = if notes_path.exists() {
        fs::read_to_string(&notes_path)?
    } else {
        println!("Warning: notes file not found at {}", notes_path.display());
        String::new()
    };

    println!("Generating interview summary for step {}...", step);
    let summary = generate_interview_summary(
        &model,
        &transcript_content,
        &notes_content,
        language.as_deref(),
    )?;

    let summary_path = dir.join(format!("INTERVIEW_{:02}_SUMMARY.md", step));
    fs::write(&summary_path, &summary)?;
    println!("Summary written: {}", summary_path.display());

    for s in config.steps.iter_mut() {
        if s.number == step {
            s.status = "completed".to_string();
            break;
        }
    }
    config.save(&dir)?;

    Ok(())
}

pub fn run_interview_progression(
    company: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    model: String,
    language: Option<String>,
) -> Result<()> {
    let interviews_dir = PathBuf::from("interviews");

    let companies: Vec<String> = if let Some(c) = company {
        let dir = company_dir(&c);
        if !dir.join("interview.json").exists() {
            return Err(anyhow!(
                "Company '{}' not initialized. Run 'promoteme interview init {}' first.",
                c,
                c
            ));
        }
        vec![c]
    } else {
        if !interviews_dir.is_dir() {
            return Err(anyhow!("No interviews directory found. Run 'promoteme interview init <company>' first."));
        }
        let mut found = Vec::new();
        for entry in fs::read_dir(&interviews_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.join("interview.json").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    found.push(name.to_string());
                }
            }
        }
        if found.is_empty() {
            return Err(anyhow!("No initialized companies found in interviews/. Run 'promoteme interview init <company>' first."));
        }
        found
    };

    if !check_ai_available(&model) {
        return Err(anyhow!(
            "AI CLI '{}' not found. Install it or specify a different model with -m.",
            model
        ));
    }

    let mut all_content = String::new();

    for company_name in &companies {
        let dir = company_dir(company_name);
        let config = InterviewConfig::load(&dir)?;

        let filtered_steps: Vec<&InterviewStep> = config
            .steps
            .iter()
            .filter(|s| {
                if let Some(ref sd) = start_date {
                    if s.date.as_str() < sd.as_str() {
                        return false;
                    }
                }
                if let Some(ref ed) = end_date {
                    if s.date.as_str() > ed.as_str() {
                        return false;
                    }
                }
                true
            })
            .collect();

        if filtered_steps.is_empty() {
            continue;
        }

        all_content.push_str(&format!("\n\n===\nCompany: {}\n===\n", company_name));

        for step in filtered_steps {
            let step_name = format!("step_{:02}", step.number);
            let title_display = step
                .title
                .as_deref()
                .map(|t| format!(" - {}", t))
                .unwrap_or_default();

            all_content.push_str(&format!(
                "\n--- Step {}{} ({})\n",
                step.number, title_display, step.date
            ));

            let summary_path = dir.join(format!("INTERVIEW_{:02}_SUMMARY.md", step.number));
            if summary_path.exists() {
                let summary = fs::read_to_string(&summary_path)?;
                all_content.push_str("Summary:\n");
                all_content.push_str(&summary);
            }

            let transcript_dir = dir.join("transcripts").join(&step_name);
            let transcripts = collect_transcripts(&transcript_dir)?;
            if !transcripts.is_empty() {
                all_content.push_str("Transcripts:\n");
                all_content.push_str(&transcripts);
            }

            let notes_path = dir.join("notes").join(format!("{}.md", step_name));
            if notes_path.exists() {
                let notes = fs::read_to_string(&notes_path)?;
                all_content.push_str("Notes:\n");
                all_content.push_str(&notes);
            }
        }
    }

    if all_content.is_empty() {
        return Err(anyhow!("No interview data found for the specified criteria."));
    }

    println!("Generating progression report...");
    let report =
        generate_progression_report(&model, &all_content, language.as_deref())?;

    println!("{}", report);
    Ok(())
}
