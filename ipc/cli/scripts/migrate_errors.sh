#!/bin/bash
# Script to help identify IPC CLI files that need error handling migration

echo "IPC CLI Error Migration Helper"
echo "=============================="
echo ""

# Find all Rust files in the commands directory
echo "Scanning for files that need error handling updates..."
echo ""

# Check for bail! usage
echo "Files using bail! macro:"
grep -r "bail!" ipc/cli/src/commands --include="*.rs" -l | sort | uniq
echo ""

# Check for anyhow! usage
echo "Files using anyhow! macro:"
grep -r "anyhow!" ipc/cli/src/commands --include="*.rs" -l | sort | uniq
echo ""

# Check for ensure! usage
echo "Files using ensure! macro:"
grep -r "ensure!" ipc/cli/src/commands --include="*.rs" -l | sort | uniq
echo ""

# Check for generic context usage
echo "Files using generic .context():"
grep -r "\.context(" ipc/cli/src/commands --include="*.rs" -l | sort | uniq
echo ""

# Count occurrences
echo "Summary:"
echo "--------"
echo "bail! occurrences: $(grep -r "bail!" ipc/cli/src/commands --include="*.rs" | wc -l)"
echo "anyhow! occurrences: $(grep -r "anyhow!" ipc/cli/src/commands --include="*.rs" | wc -l)"
echo "ensure! occurrences: $(grep -r "ensure!" ipc/cli/src/commands --include="*.rs" | wc -l)"
echo ".context() occurrences: $(grep -r "\.context(" ipc/cli/src/commands --include="*.rs" | wc -l)"
echo ""

echo "To start migration:"
echo "1. Pick a file from the lists above"
echo "2. Add 'use crate::errors::{CliError, ...};' to imports"
echo "3. Replace generic errors with specific error types"
echo "4. Test the command to ensure error messages are user-friendly"
echo "5. Refer to ERROR_HANDLING_GUIDE.md for detailed patterns"