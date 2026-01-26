# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Promoteme is a Rust CLI tool that generates "brag documents" - summaries of GitHub or Azure DevOps contributions for performance reviews. It fetches PRs via CLI tools, analyzes them, and uses AI (Claude by default) to synthesize impact narratives.

## Running the CLI

```bash
# Build the project
cargo build --release

# Basic usage - GitHub (default)
./target/release/promoteme generate

# With date filters
./target/release/promoteme generate --start-date 2024-01-01 --end-date 2024-06-30

# Specify output language
./target/release/promoteme generate -l Portuguese

# Use specific AI model
./target/release/promoteme generate -m claude@claude-opus-4-5

# Use Azure DevOps as source
./target/release/promoteme generate --source azuredevops --org myorg

# Azure DevOps with project/repo filter
./target/release/promoteme generate --source azuredevops --org myorg --repo myproject/myrepo
```

## Dependencies

Required tools that must be installed:

For GitHub:
- `gh` (GitHub CLI) - must be authenticated via `gh auth login`

For Azure DevOps:
- `az` (Azure CLI) - must be authenticated via `az login`
- Azure DevOps extension: `az extension add --name azure-devops`

Common:
- AI CLI (default: `claude`) - for final document synthesis

## Architecture

Rust CLI with modular provider system supporting multiple PR sources:

1. **Provider Layer**: Abstracted PR fetching via `PrProvider` trait
   - `GitHubProvider`: Uses `gh` CLI
   - `AzureDevOpsProvider`: Uses `az repos pr` commands
2. **Processing**: Groups PRs by repo, analyzes each (scope, impact, risk, tests)
3. **Consolidation**: AI synthesizes per-repo reports into final `README.md`

**Output structure:**
```
{username}/
  ├── {repo1}.md      # Raw PR analysis
  ├── {repo2}.md
  └── README.md       # AI-generated executive summary
```

## Key Files

- `src/main.rs` - Entry point and orchestration
- `src/cli.rs` - CLI argument parsing
- `src/provider.rs` - `PrProvider` trait definition
- `src/github.rs` - GitHub provider implementation
- `src/azuredevops.rs` - Azure DevOps provider implementation
- `src/processor.rs` - PR processing logic
- `src/models.rs` - Data structures
- `src/ai.rs` - AI integration for document generation

## Code Style Rules

1. Avoid comments
2. Never use emojis
3. Avoid dead code
4. Use Rust best practices
5. Always validate user input
