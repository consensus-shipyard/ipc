#!/usr/bin/env bash

REPORT_FILE="./report.md"

if [ ! -f $REPORT_FILE ]; then
    echo "Report file not found."
    exit 1;
fi

# Check if one of `| Critical | 0 |`, `| High | 0 |`, or `| Medium | 0 |` line exist in the report.
zero_findings=`(grep -e "Critical\s*|\s*0" $REPORT_FILE && grep -e  "High\s*|\s*0" $REPORT_FILE && grep -e  "Medium\s*|\s*0" $REPORT_FILE) | wc -l`

if [ $zero_findings -eq 3 ]; then
    echo "No critical or high issues found"
    exit 0
else
    echo "Critical, high, or medium issue found".
    echo "Check $REPORT_FILE for more information".
    echo "Printing here..."
    cat $REPORT_FILE
    exit 1;
fi
