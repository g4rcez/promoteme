# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Promoteme is a Bash CLI tool that generates "brag documents" - summaries of GitHub contributions for performance reviews. It fetches PRs via GitHub API, analyzes them, and uses AI (Claude by default) to synthesize impact narratives.

## Running the CLI

```bash
# Make executable (first time)
chmod +x promoteme.sh

# Basic usage - all contributions
./promoteme.sh

# With date filters
./promoteme.sh --start-date 2024-01-01 --end-date 2024-06-30

# Specify output language
./promoteme.sh -l Portuguese

# Use specific AI model
./promoteme.sh -m claude@claude-opus-4-5
```

## Dependencies

Required tools that must be installed:
- `gh` (GitHub CLI) - must be authenticated via `gh auth login`
- `jq` - JSON processor
- AI CLI (default: `claude`) - for final document synthesis

## Architecture

Single-file Bash script (`promoteme.sh`) with three phases:

1. **Data Collection**: Fetches all PRs by current user via `gh api search/issues`
2. **Processing**: Groups PRs by repo, analyzes each (scope, impact, risk, tests)
3. **Consolidation**: AI synthesizes per-repo reports into final `README.md`

**Output structure:**
```
{github-username}/
  ├── {repo1}.md      # Raw PR analysis
  ├── {repo2}.md
  └── README.md       # AI-generated executive summary
```

## Key Files

- `promoteme.sh` - Main CLI script
- `PROMPT.txt` - AI prompt template for generating summaries
- `EXAMPLES.md` - Usage examples with date filtering

## Code Style Rules

1. Avoid comments
2. Never use emojis
3. Avoid dead code
4. Use Rust best practices
5. Always validate user input
