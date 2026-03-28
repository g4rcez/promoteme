use anyhow::{bail, Result};
use std::process::Command;

const PROMPT_TEMPLATE: &str = include_str!("prompt.txt");

fn parse_model_spec(model: &str) -> (&str, Option<&str>) {
    match model.split_once('@') {
        Some((cli, model_name)) => (cli, Some(model_name)),
        None => (model, None),
    }
}

pub fn check_ai_available(model: &str) -> bool {
    let (cli, _) = parse_model_spec(model);
    Command::new("which")
        .arg(cli)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn invoke_ai(model: &str, prompt: &str) -> Result<String> {
    let (cli, model_name) = parse_model_spec(model);

    let mut cmd = Command::new(cli);
    if let Some(m) = model_name {
        cmd.args(["--model", m]);
    }
    cmd.args(["-p", prompt]);

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_output = if !stderr.is_empty() { stderr } else { stdout };
        bail!("AI invocation failed: {}", error_output);
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

pub fn generate_team_document(
    model: &str,
    members_content: &str,
    language: Option<&str>,
) -> Result<String> {
    let prompt_body = concat!(
        "You are a senior engineering manager writing a team performance overview for a performance cycle or retrospective. ",
        "You have quantitative metrics (PRs merged, open, reviews given, test coverage, PR sizing, score) and qualitative PR data ",
        "(titles, sizes, repositories) for each team member. Produce a structured markdown document with exactly these sections:\n\n",
        "## Executive Summary\n",
        "Two to three sentences on overall team health, delivery velocity, and the single most important signal from this period.\n\n",
        "## Team Delivery Patterns\n",
        "Analyze patterns visible across all members:\n",
        "- Delivery velocity: merge rate, PR throughput, and pacing signals\n",
        "- PR sizing discipline: distribution of small vs. large changesets and what it reveals about workflow habits\n",
        "- Test coverage culture: proportion of PRs with test files; call out outliers in either direction\n",
        "- Documentation practice: proportion of PRs that include documentation changes; patterns of README or docs/ updates\n",
        "- Scope of work: cross-repo vs. single-repo contributors; depth vs. breadth tradeoffs\n\n",
        "## Code Review Culture\n",
        "Treat review behavior as a proxy for collaboration and knowledge sharing:\n",
        "- Review-to-authored-PR ratio per member: who actively unblocks others vs. who focuses only on delivery\n",
        "- Identify potential knowledge silos (members whose review activity is absent or narrowly scoped)\n",
        "- Note asymmetry: if some members consistently review while others rarely do, call it out explicitly\n",
        "- Distinguish high-volume reviewers from members with zero or near-zero review counts\n",
        "- Review quality: ratio of substantive reviews (Quality Reviews) to total reviews; flag members whose reviews are primarily low-effort approvals\n\n",
        "## Technical Decision-Making and Workflow Signals\n",
        "Infer decision-making patterns from the PR data:\n",
        "- Large PRs (>500 lines): could indicate insufficient task decomposition, big-bang refactors, or decisive ownership of complex problems — reason about which applies\n",
        "- Small, frequent PRs: iterative workflow, good PR hygiene, or risk aversion — consider the context\n",
        "- PRs with no test files appearing across multiple members may indicate a systemic gap rather than an individual habit\n",
        "- Cross-repo work signals architectural awareness or ownership of shared dependencies\n",
        "- Members with high open-PR counts may be bottlenecked on reviews or working on long-horizon initiatives\n\n",
        "## Per-Member Narratives\n",
        "For each team member write two to four sentences from a leadership perspective. Cover:\n",
        "- What their quantitative data and PR titles reveal about their focus and working style\n",
        "- Whether their output is above, at, or below expectations for their level and role (when provided)\n",
        "- One concrete strength observable in the data and, if relevant, one growth area\n",
        "- Do not rank members against each other — assess each relative to their own level and role expectations\n\n",
        "Where a Level and Role are provided: an entry-level engineer delivering complex cross-cutting work warrants stronger recognition ",
        "than a senior doing the same. A senior or tech lead with low review activity is a concern regardless of their PR count. ",
        "A tech lead or architect with no cross-repo work may be operating below scope.\n\n",
        "## Team Health Indicators\n",
        "Summarize systemic signals:\n",
        "- Bus factor risk: is contribution highly concentrated in one or two members?\n",
        "- Collaboration density: are reviews spread across the team or clustered?\n",
        "- Knowledge distribution: which repositories have single-contributor risk?\n",
        "- Growth signals: are entry-level or mid-level members expanding their scope or review participation?\n",
        "- Any pattern that warrants direct attention from leadership\n\n",
        "## Recommendations\n",
        "Two to four concrete, actionable suggestions for the manager. Ground each recommendation in a specific observed pattern from the data. ",
        "Avoid generic advice — every recommendation must be traceable to something in the input.\n\n",
    );
    let mut prompt = prompt_body.to_string();

    if let Some(lang) = language {
        prompt.push_str(&format!("Provide the entire output in {}.\n\n", lang));
    }

    prompt.push_str("--- TEAM DATA ---\n\n");
    prompt.push_str(members_content);

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
