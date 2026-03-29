# promoteme

A CLI that helps you write brag documents by analyzing your GitHub contributions.

## Features

- Automatic PR fetching via GitHub CLI
- Repository grouping and analysis
- AI-powered document generation (Claude/Gemini) with graceful fallback when AI CLI is not installed
- Multi-language output support
- Personal notes integration
- Date range filtering
- Organization and repository filtering
- Team mode: analyze all members of an org and generate a team performance overview
- Team scoring: quantitative metrics (PRs, quality reviews, commits, test/doc coverage)
- Team config (`team.json`): assign seniority levels and roles for level-aware AI evaluation

## Prerequisites

- [GitHub CLI](https://cli.github.com/) (`gh`) installed and authenticated via `gh auth login`
- AI CLI for document synthesis:
  - [Claude CLI](https://docs.anthropic.com/en/docs/claude-code) (default)
  - or [Gemini CLI](https://github.com/google-gemini/gemini-cli)

## Installation

### Via Cargo (crates.io)
```bash
cargo install promoteme
```

### From source
```bash
git clone https://github.com/g4rcez/promoteme
cd promoteme
cargo install --path .
```

## Usage

```
promoteme generate [OPTIONS]

Options:
  --start-date    Start date YYYY-MM-DD (default: 6 months ago)
  --end-date      End date YYYY-MM-DD (default: today)
  --org           Filter by organization(s), comma-separated (e.g. org1,org2)
  --repo          Filter by repo(s), comma-separated (e.g. owner/repo1,owner/repo2)
  -l, --language  Output language (English, Portuguese, etc.)
  -m, --model     AI model: claude (default), gemini. Use cli@model format for specific model.
  --notes         Directory with personal notes (.md/.txt)
  --cwd           Custom output directory (default: artifacts/{username}_{timestamp}Z)
  --team          Enable team mode (requires --members or --org)
  --members       Comma-separated GitHub usernames for team mode
  --setup         Write team.json template and exit (requires --team)
```

### Examples

Basic usage (last 6 months):
```bash
promoteme generate
```

With date filters:
```bash
promoteme generate --start-date 2024-01-01 --end-date 2024-06-30
```

Filter by organization:
```bash
promoteme generate --org my-company
```

Filter by specific repositories:
```bash
promoteme generate --repo owner/repo1,owner/repo2
```

Output in Portuguese:
```bash
promoteme generate -l Portuguese
```

Using Gemini instead of Claude:
```bash
promoteme generate -m gemini
```

Using specific model:
```bash
promoteme generate -m claude@claude-opus-4-5
```

Include personal notes:
```bash
promoteme generate --notes ~/my-notes
```

Custom output directory:
```bash
promoteme generate --cwd /path/to/output
```

### Team mode

Analyze all members of an organization and generate a team performance overview:
```bash
promoteme generate --team --org my-company --start-date 2025-01-01
```

Analyze a specific set of members:
```bash
promoteme generate --team --members alice,bob,carol --start-date 2025-01-01
```

#### Level-aware evaluation with `team.json`

Generate a `team.json` config template pre-populated with all org members:
```bash
promoteme generate --team --org my-company --setup
```

Edit `{org_name}/team.json` to set each member's seniority level and role:
```json
{
  "members": {
    "alice": { "level": "senior", "role": "Backend Engineer" },
    "bob":   { "level": "tech_lead", "role": null },
    "carol": { "level": "entrylevel", "role": "Frontend Engineer" }
  }
}
```

Valid levels: `entrylevel`, `mid`, `senior`, `tech_lead`, `specialist`, `architect`, `manager`

When `{org_name}/team.json` is present, `promoteme` loads it automatically and the AI evaluates each member's contributions relative to their expected level.

Team mode generates:
- `{member}.md` — per-member contribution report
- `SCORES.md` — ranked table with score, merged PRs, quality reviews, commits, and avg PR size
- `README.md` — AI-generated team performance overview

## Tech stack

- Rust
- Github CLI - https://cli.github.com/

## What is a Brag Document?

A brag document is a running record of your professional accomplishments. It serves as a personal changelog of your work contributions.

### When to use

- Performance reviews
- Promotion discussions
- Job interviews
- Salary negotiations
- Updating your resume

### Tips for maintaining one

- Update it regularly (weekly or bi-weekly)
- Include quantifiable impact when possible
- Document both technical and non-technical contributions
- Keep it factual and specific

### What to include

- Pull requests and code contributions
- Code reviews and mentoring
- Documentation improvements
- Bug fixes and incidents resolved
- Process improvements
- Cross-team collaboration
- Knowledge sharing and presentations

## References

- https://jvns.ca/blog/brag-documents/
- https://www.reddit.com/r/webdev/comments/19dtf6r/why_a_brag_doc_is_indispensable/
