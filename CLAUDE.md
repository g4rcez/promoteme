# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Promoteme is a Rust CLI tool that generates "brag documents" - summaries of GitHub contributions for performance reviews. It fetches PRs via GitHub CLI, analyzes them, and uses AI (Claude by default) to synthesize impact narratives. It also supports a team mode for engineering managers to generate level-aware team performance overviews.

## Running the CLI

```bash
# Build
cargo build

# Basic usage - all contributions
./target/debug/promoteme generate

# With date filters
./target/debug/promoteme generate --start-date 2024-01-01 --end-date 2024-06-30

# Specify output language
./target/debug/promoteme generate -l Portuguese

# Use specific AI model
./target/debug/promoteme generate -m claude@claude-opus-4-5

# Team mode - analyze all members of an org
./target/debug/promoteme generate --team --org my-company --start-date 2025-01-01

# Team setup - generate team.json template
./target/debug/promoteme generate --team --org my-company --setup
```

## Dependencies

Required tools that must be installed:
- `gh` (GitHub CLI) - must be authenticated via `gh auth login`
- AI CLI (default: `claude`) - for final document synthesis

## Architecture

Rust CLI with multiple modules:

1. **Data Collection** (`src/github.rs`): Fetches PRs and reviews via `gh` CLI
2. **Processing** (`src/processor.rs`, `src/team.rs`): Groups PRs by repo, computes member stats
3. **AI synthesis** (`src/ai.rs`): Generates final documents via AI CLI
4. **Config** (`src/config.rs`): Loads/writes `team.json` for level-aware team evaluation

**Output structure (individual mode):**
```
artifacts/{username}_{timestamp}/
  ├── {repo1}.md        # Per-repo PR analysis
  ├── {repo2}.md
  ├── NOTES_SUMMARY.md  # (optional) AI summary of personal notes
  └── README.md         # AI-generated executive summary
```

**Output structure (team mode):**
```
{org_name}/{YYYY_MM_DD_HH_MM}/
  ├── {member1}.md      # Per-member contribution report
  ├── {member2}.md
  ├── SCORES.md         # Quantitative scores table
  └── README.md         # AI-generated team performance overview
```

## Key Files

- `src/main.rs` - Entry point, CLI dispatch, `run_generate`, `run_team_generate`, `run_team_setup`, `resolve_members`
- `src/cli.rs` - Clap CLI definitions
- `src/ai.rs` - AI invocation and prompt construction
- `src/github.rs` - GitHub API calls via `gh` CLI
- `src/config.rs` - `TeamConfig`, `MemberLevel`, `generate_setup_file`, `load_team_config`
- `src/team.rs` - Member stats and report generation
- `src/processor.rs` - PR processing and repo grouping
- `src/prompt.txt` - AI prompt template for individual brag docs
- `EXAMPLES.md` - Usage examples

## Team Config (`team.json`)

Stored at `{org_name}/team.json` (written by `--setup`, auto-detected by generate). Covered by `.gitignore`. The AI uses level/role context to evaluate contributions relative to seniority expectations.

```json
{
  "members": {
    "alice": { "level": "senior", "role": "Backend Engineer" },
    "bob":   { "level": "tech_lead", "role": null }
  }
}
```

Valid levels: `entrylevel`, `mid`, `senior`, `tech_lead`, `specialist`, `architect`, `manager`

Generate template: `promoteme generate --team --org <org> --setup`

## Code Style Rules

1. Avoid comments
2. Never use emojis
3. Avoid dead code
4. Use Rust best practices
5. Always validate user input
