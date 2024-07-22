#!/bin/sh
set -eu

# Path to the report file
REPORT_FILE="./report.json"

# List of severities that make us fail
SEVERITIES="critical high medium"

# Function to check if a vulnerability title should be ignored
ignore_title() {
    case "$1" in
        "Centralization Risk for trusted owners") return 0 ;;
        # Add more titles to ignore here, one per line
        *) return 1 ;;
    esac
}

# Function to check if a specific vulnerability should be ignored
ignore_specific() {
    case "$1" in
        "src/lib/LibDiamond.sol:204:Unprotected initializer") return 0 ;;
        "src/lib/LibDiamond.sol:203:Unprotected initializer") return 0 ;;
        *) return 1 ;;
    esac
}

# Read vulnerabilities from the report
read_vulnerabilities() {
    level="$1"
    jq -c ".${level}_issues.issues[]? // empty" "$REPORT_FILE"
}

# Main function to process the report
process_report() {
    has_vulnerabilities=0

    for level in $SEVERITIES; do
        read_vulnerabilities "$level" | while IFS= read -r vulnerability; do
            title=$(printf '%s' "$vulnerability" | jq -r ".title")
            path=$(printf '%s' "$vulnerability" | jq -r ".instances[].contract_path")
            line=$(printf '%s' "$vulnerability" | jq -r ".instances[].line_no")
            specific_key="${path}:${line}:${title}"

            if ignore_specific "$specific_key"; then
                printf "Ignoring specific vulnerability: %s at %s line %s\n" "$title" "$path" "$line"
            elif ignore_title "$title"; then
                printf "Ignoring vulnerability by title: %s at %s line %s\n" "$title" "$path" "$line"
            else
                printf "Found %s vulnerability: %s at %s line %s\n" "$level" "$title" "$path" "$line"
                has_vulnerabilities=1
            fi
        done
    done

    return $has_vulnerabilities
}

# Process the report and exit with the code returned by process_report
process_report
exit $?