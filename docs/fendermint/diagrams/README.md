# Diagrams

This directory contains [Mermaid](https://mermaid.js.org/) diagrams which are turned into images ready to be embedded into docs.

To render the images, run the following command:

```shell
make diagrams
```

## Prerequisites

The build process uses the Mermaid CLI tool which requires Node.js. The Makefile will automatically install the required dependencies locally.

## Automation

Adding the following script to `.git/hooks/pre-commit` automatically renders and checks in the images when we commit changes to their source diagrams. CI should also check that there are no uncommitted changes.

```bash
#!/usr/bin/env bash

# If any command fails, exit immediately with that command's exit status
set -eo pipefail

# Redirect output to stderr.
exec 1>&2

if git diff --cached --name-only  --diff-filter=d | grep .mmd
then
  make diagrams
  git add docs/fendermint/diagrams/*.png
fi
```

## Cleaning Up

To clean up generated files and dependencies:

```shell
make clean
```
