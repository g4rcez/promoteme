# Hireme: Interview Tracking Feature

## Context

Job seekers need a structured way to track their interview processes across multiple companies, capture what was said (via external teleprompter transcription), reflect on their own performance (via personal notes), and get AI-powered feedback on their progression over time. This feature adds an `interview` subcommand to the existing promoteme CLI.

## Directory Structure

```
interviews/
  {company}/
    transcripts/
      step_{NN}/           # external teleprompter writes .md files here
    notes/
      step_{NN}.md         # user's personal notes per step
    INTERVIEW_{NN}_SUMMARY.md  # AI-generated summary per step
    interview.json         # metadata and step tracking
```

### interview.json

```json
{
  "company": "acme",
  "created_at": "2026-03-29",
  "steps": [
    {
      "number": 1,
      "title": "Phone Screen",
      "date": "2026-03-29",
      "status": "active"
    }
  ]
}
```

Step status values: `active`, `completed`.

## CLI Commands

All commands live under `promoteme interview <subcommand>`.

### `init <company>`

- Creates `interviews/{company}/` with `transcripts/`, `notes/` subdirs and `interview.json`
- Fails if directory already exists (prevents accidental overwrites)

### `new <step> --company <company> [--title TITLE] [--start-teleprompter]`

- `<step>`: integer step number
- Validates company directory exists (must run `init` first)
- Creates `transcripts/step_{NN}/` directory
- Creates `notes/step_{NN}.md` with a template containing title and date
- Appends step entry to `interview.json`
- `--start-teleprompter`: prints the path where the external teleprompter should write transcription files (`transcripts/step_{NN}/`)

### `summarize --company <company> --step <N> [-m MODEL] [-l LANGUAGE]`

- Reads all `.md` files from `transcripts/step_{NN}/`
- Reads `notes/step_{NN}.md`
- Sends combined content to AI via `invoke_ai` with a structured prompt requesting:
  - Key topics discussed
  - Questions asked and answers given
  - Technical concepts covered
  - Areas of strength and weakness
- Writes output to `INTERVIEW_{NN}_SUMMARY.md` in the company directory
- Marks the step as `completed` in `interview.json`

### `progression [--company <company>] [--start-date DATE] [--end-date DATE] [-m MODEL] [-l LANGUAGE]`

- If `--company` specified, analyzes only that company. Otherwise, all companies under `interviews/`.
- Reads all `INTERVIEW_{NN}_SUMMARY.md` files, transcripts, and notes within scope
- Date filtering based on step dates in `interview.json`
- Sends to AI with a prompt requesting:
  - Progress timeline and growth areas
  - Recurring mistakes and patterns
  - Concrete improvement suggestions
  - Comparison across interview steps/companies
- Outputs to stdout

## Code Organization

### New Files

- `src/interview.rs` -- all interview logic: `run_interview_init`, `run_interview_new`, `run_interview_summarize`, `run_interview_progression`
- `src/interview_prompt.txt` -- AI prompt template for step summaries
- `src/progression_prompt.txt` -- AI prompt template for progression analysis

### Modified Files

- `src/cli.rs` -- add `Commands::Interview` variant containing a nested `InterviewCommands` enum with `Init`, `New`, `Summarize`, `Progression` variants
- `src/main.rs` -- add match arm for `Commands::Interview` dispatching to `interview.rs` functions
- `src/ai.rs` -- add `generate_interview_summary(model, transcript_content, notes_content, language)` and `generate_progression_report(model, all_content, language)`

### Patterns to Reuse

- `ai::invoke_ai` and `ai::check_ai_available` for AI integration
- `fs::create_dir_all` / `fs::write` for file I/O
- `serde_json` for `interview.json` serialization
- `anyhow::Result` for error handling
- `notes::collect_notes` pattern for reading .md files from a directory

### Data Structures (in interview.rs)

- `InterviewConfig` -- deserializes `interview.json` (company, created_at, steps vec)
- `InterviewStep` -- individual step (number, title, date, status)

## Verification

1. `cargo build` -- must compile without errors
2. `promoteme interview init testco` -- creates `interviews/testco/` with correct structure
3. `promoteme interview new 1 --company testco --title "Phone Screen"` -- creates step dirs/files, updates interview.json
4. Place sample .md files in `interviews/testco/transcripts/step_01/`
5. Write notes in `interviews/testco/notes/step_01.md`
6. `promoteme interview summarize --company testco --step 1` -- generates INTERVIEW_01_SUMMARY.md
7. `promoteme interview progression --company testco` -- outputs progression analysis
