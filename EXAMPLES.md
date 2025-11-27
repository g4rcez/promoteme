# devitrine Usage Examples

This document provides examples for using the `devitrine.sh` script to generate brag documents.

## Prerequisites

Before running `devitrine.sh`, ensure you have the following installed and configured:

1.  **GitHub CLI (`gh`)**: Follow the official installation instructions for your OS.
    *   Authenticate your `gh` CLI by running `gh auth login`.
2.  **`jq`**: A lightweight and flexible command-line JSON processor. Most Linux distributions and macOS (via Homebrew) can install it easily.

## Making the Script Executable

If you haven't already, make the `devitrine.sh` script executable:

```bash
chmod +x devitrine.sh
```

## Basic Usage

Run the script without any date filters to retrieve all your contributions across all time (or as far back as GitHub's API allows for search queries).

```bash
./devitrine.sh
```

## Filter by a Specific Date Range

Generate a brag document for contributions made within a specific period using `--start-date` and `--end-date`. Dates should be in `YYYY-MM-DD` format.

```bash
./devitrine.sh --start-date 2023-01-01 --end-date 2023-03-31
```

## Filter from a Start Date Onwards

Generate a brag document for all contributions from a specific start date up to the present.

```bash
./devitrine.sh --start-date 2024-06-01
```

## Filter Up to an End Date

Generate a brag document for all contributions from the beginning of time up to a specific end date.

```bash
./devitrine.sh --end-date 2023-12-31
```

## Example Output

The script generates output in Markdown format, which can be easily viewed in a Markdown viewer or converted to other formats.

```markdown
# Brag Document
## Period: created:>=2024-06-01

## Repositories Contributed To
- octocat/Spoon-Knife
- my-org/my-project

## Contributions Summary
### octocat/Spoon-Knife
- [merged] [Update README.md with new instructions](https://github.com/octocat/Spoon-Knife/pull/123) - 2024-06-15T10:30:00Z
- [closed] [Fix typo in CONTRIBUTING.md](https://github.com/octocat/Spoon-Knife/pull/124) - 2024-07-01T14:00:00Z

### my-org/my-project
- [open] [Feature: Implement user profile editing](https://github.com/my-org/my-project/pull/456) - 2024-06-20T09:00:00Z
- [merged] [Bugfix: Resolve authentication issue](https://github.com/my-org/my-project/pull/455) - 2024-06-10T11:45:00Z
```
