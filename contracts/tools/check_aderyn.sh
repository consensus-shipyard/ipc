#!/usr/bin/env bash
set -eu
set -o pipefail

# Path to the report file
REPORT_FILE="./report.json"

# List of severities that make us fail
SEVERITIES=(critical high medium)

# List of vulnerability titles to ignore
IGNORE_TITLES=("Centralization Risk for trusted owners")

containsElement() {
  local e match="$1"
  shift
  for e; do [[ "$e" == "$match" ]] && return 0; done
  return 1
}

# Read vulnerabilities from the report
readVulnerabilities() {
  level="$1"
  jq -c --argjson ignoreTitles "$(printf '%s\n' "${IGNORE_TITLES[@]}" | jq -R . | jq -s .)" ".${level}_issues.issues[] | select(.title as \$title | \$ignoreTitles | index(\$title) | not)" $REPORT_FILE
}

# Main function to process the report
processReport() {
  local hasVulnerabilities=0

  for level in ${SEVERITIES[@]}; do
    while IFS= read -r vulnerability; do
      title=$(echo "$vulnerability" | jq -r ".title")
      echo "Found $level vulnerability: $title"
      hasVulnerabilities=1
    done < <(readVulnerabilities "$level")
  done

  return $hasVulnerabilities
}

# Process the report and exit with the code returned by processReport
processReport
exit $?