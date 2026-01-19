use anyhow::{bail, Result};
use std::process::Command;

const PROMPT_TEMPLATE: &str = include_str!("prompt.txt");

/// Check if AI CLI is available
pub fn check_ai_available(model: &str) -> bool {
    Command::new("which")
        .arg(model)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Invoke AI CLI with a prompt
pub fn invoke_ai(model: &str, prompt: &str) -> Result<String> {
    let output = Command::new(model)
        .args(["-p", prompt])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("AI invocation failed: {}", stderr);
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// Generate notes summary using AI
pub fn generate_notes_summary(model: &str, notes_content: &str, language: Option<&str>) -> Result<String> {
    let mut prompt = "Summarize these personal notes about team contributions, leadership, and non-code impact. Focus on: collaboration, mentorship, process improvements, cross-team work. Output in markdown.".to_string();

    if let Some(lang) = language {
        prompt.push_str(&format!(" Output in {}.", lang));
    }

    prompt.push_str(notes_content);

    invoke_ai(model, &prompt)
}

/// Generate final consolidated document
pub fn generate_final_document(
    model: &str,
    repo_content: &str,
    notes_content: Option<&str>,
    language: Option<&str>,
) -> Result<String> {
    let mut prompt = PROMPT_TEMPLATE.to_string();

    prompt.push_str("\n\nTask: Synthesize the following repository summaries into a single, cohesive Brag Document for the entire period. Highlight the overall impact across all projects.");

    prompt.push_str("\n\nAdditionally, after the main executive summary and highlights, please provide a dedicated section titled \"Repository Breakdown\". For EACH repository found in the input, provide a summary using this format:\n");
    prompt.push_str("### [Project Name]\n");
    prompt.push_str("- **Key Features:** List 2-3 main features or changes delivered.\n");
    prompt.push_str("- **Business Value:** Explain the tangible benefit (e.g., \"Improved user experience,\" \"Reduced build time\").\n");
    prompt.push_str("- **Technical Stack:** inferred from the context (e.g., React, Next.js, Go).\n");

    if let Some(lang) = language {
        prompt.push_str(&format!("\n\nPlease provide the output in {}.", lang));
    }

    prompt.push_str(repo_content);

    if let Some(notes) = notes_content {
        prompt.push_str("\n\n---\n");
        prompt.push_str("PERSONAL NOTES (non-code contributions, team impact, leadership):\n");
        prompt.push_str(notes);
    }

    invoke_ai(model, &prompt)
}

pub fn translate_report(model: &str, content: &str, language: &str) -> Result<String> {
    let prompt = format!(
        "Translate this markdown to {}. Keep markdown formatting and structure intact:\n\n{}",
        language, content
    );
    invoke_ai(model, &prompt)
}

/// Fallback: concatenate all reports into a single document
pub fn concatenate_reports(reports: &[(String, String)], dir_suffix: &str) -> String {
    let mut content = format!("# Brag documents - {}\n", dir_suffix);

    for (_, report) in reports {
        content.push('\n');
        content.push_str(report);
    }

    content
}
