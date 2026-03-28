# promoteme Usage Examples

## Prerequisites

- [GitHub CLI](https://cli.github.com/) (`gh`) installed and authenticated via `gh auth login`
- AI CLI for document synthesis: [Claude CLI](https://docs.anthropic.com/en/docs/claude-code) (default)

## Individual Mode

Basic usage (last 6 months):
```bash
promoteme generate
```

Filter by date range:
```bash
promoteme generate --start-date 2024-01-01 --end-date 2024-06-30
```

From a start date onwards:
```bash
promoteme generate --start-date 2024-06-01
```

Filter by organization:
```bash
promoteme generate --org my-company
```

Output in Portuguese with a specific model:
```bash
promoteme generate -l Portuguese -m claude@claude-opus-4-5
```

Include personal notes (`.md`/`.txt` files):
```bash
promoteme generate --notes ~/my-notes
```

Custom output directory:
```bash
promoteme generate --cwd ./my-brag-2025
```

## Team Mode

Analyze all members of an org (last 6 months):
```bash
promoteme generate --team --org my-company
```

Analyze specific members with a date filter:
```bash
promoteme generate --team --members alice,bob,carol --start-date 2025-01-01
```

### Setup: level-aware evaluation

Generate a `team.json` template with all org members defaulting to `entrylevel`:
```bash
promoteme generate --team --org my-company --setup
```

Edit `team.json` to reflect each member's actual level and role:
```json
{
  "members": {
    "alice": { "level": "senior", "role": "Backend Engineer" },
    "bob":   { "level": "tech_lead", "role": null },
    "carol": { "level": "entrylevel", "role": "Frontend Engineer" },
    "dave":  { "level": "architect", "role": "Platform" },
    "eve":   { "level": "manager", "role": "Engineering Manager" }
  }
}
```

Valid levels: `entrylevel`, `mid`, `senior`, `tech_lead`, `specialist`, `architect`, `manager`

Then run the team report — `team.json` is auto-detected:
```bash
promoteme generate --team --org my-company --start-date 2025-01-01
```

## Output Structure

### Individual mode
```
artifacts/{username}_{timestamp}/
  {repo1}.md        # Per-repo PR analysis
  {repo2}.md
  README.md         # AI-generated executive summary
  NOTES_SUMMARY.md  # (optional) AI summary of personal notes
```

### Team mode
```
artifacts/team_{timestamp}/
  {member1}.md      # Per-member contribution report
  {member2}.md
  SCORES.md         # Quantitative scores table
  README.md         # AI-generated team performance overview
```

> All output is written under `artifacts/` and ignored by git.
> Use `--cwd` to override the output path entirely (no prefix is added).
