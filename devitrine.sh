#!/bin/bash

# devitrine.sh
# CLI to generate brag documents based on GitHub contributions.

set -e

# Default configuration
PER_PAGE=30
DATE_FILTER=""

# check dependencies
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI ('gh') is not installed."
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "Error: jq is not installed. Please install it to use this script."
    exit 1
fi

# check auth
if ! gh auth status &> /dev/null; then
    echo "Error: You are not logged into GitHub CLI. Run 'gh auth login' first."
    exit 1
fi

# Get current user
CURRENT_USER=$(gh api user --jq .login)
if [[ -z "$CURRENT_USER" ]]; then
    echo "Error: Could not retrieve current GitHub user."
    exit 1
fi

function print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --start-date DATE   Start date (YYYY-MM-DD) for contributions."
    echo "  --end-date DATE     End date (YYYY-MM-DD) for contributions."
    echo "  --help              Show this help message."
}

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --start-date) START_DATE="$2"; shift ;;
        --end-date) END_DATE="$2"; shift ;;
        --help) print_usage; exit 0 ;;
        *) echo "Unknown parameter passed: $1"; print_usage; exit 1 ;; 
    esac
    shift
done

# Construct date filter
if [[ -n "$START_DATE" && -n "$END_DATE" ]]; then
    DATE_FILTER="created:${START_DATE}..${END_DATE}"
elif [[ -n "$START_DATE" ]]; then
    DATE_FILTER="created:>=${START_DATE}"
elif [[ -n "$END_DATE" ]]; then
    DATE_FILTER="created:<=${END_DATE}"
fi

# Function to fetch PRs
function fetch_prs() {
    echo "Fetching PRs for user: $CURRENT_USER..." >&2
    
    QUERY="author:$CURRENT_USER type:pr"
    if [[ -n "$DATE_FILTER" ]]; then
        QUERY="$QUERY $DATE_FILTER"
    fi

    # Using gh api with pagination
    # We utilize --paginate to automatically traverse all pages
    # We set per_page to 30 as requested
    # Explicitly force GET method
    gh api -X GET search/issues \
        -f q="$QUERY" \
        --paginate \
        -f per_page=$PER_PAGE \
        --jq '.items[] | {title: .title, url: .html_url, repo: .repository_url, created_at: .created_at, state: .state}'
}

# Main execution
echo "# Brag Document"
if [[ -n "$DATE_FILTER" ]]; then
    echo "## Period: $DATE_FILTER"
fi
echo ""

# We capture the output of fetch_prs into a temporary file or variable to process it.
# Since we need to group by repo, saving to a temp file is safer for large datasets.
TMP_FILE=$(mktemp)
fetch_prs > "$TMP_FILE"

# Check if we found anything
if [[ ! -s "$TMP_FILE" ]]; then
    echo "No contributions found for the specified criteria."
    rm "$TMP_FILE"
    exit 0
fi

# Extract unique repositories
# .repo is a full URL like "https://api.github.com/repos/owner/repo". We want "owner/repo".
echo "## Repositories Contributed To"
cat "$TMP_FILE" | jq -r '.repo' | sort | uniq | sed 's|https://api.github.com/repos/||' | while read -r repo; do
    echo "- $repo"
done

echo ""
echo "## Contributions Summary"

# Process per repository
# We need to read the temp file again or use jq to group.
# Using jq to group by repo is efficient.
# Note: We must ensure the jq script string is properly quoted to preserve newlines and logic.
cat "$TMP_FILE" | jq -s -r --argjson per_page "$PER_PAGE" '
    # Clean up repo URL to just name
    map(.repo |= sub("https://api.github.com/repos/"; "")) |
    # Group by repository
    group_by(.repo) |
    .[] |
    "### " + .[0].repo + "\n" +
    (map("- [" + .state + "] [" + .title + "](" + .url + ") - " + .created_at) | join("\n")) +
    "\n"
'

# Cleanup
rm "$TMP_FILE"