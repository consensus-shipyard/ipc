FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    python3 python3-pip python3-venv \
    curl wget jq git \
    build-essential \
    nodejs npm \
    && rm -rf /var/lib/apt/lists/*

RUN pip3 install --break-system-packages \
    anthropic requests httpx pandas numpy \
    matplotlib seaborn rich tabulate

# Anthropic Messages API tuning (agent.py reads these; override at run time if needed).
ENV ANTHROPIC_MODEL=claude-sonnet-4-5
ENV ANTHROPIC_MAX_TOKENS=4096
ENV CONTEXT_LIMIT=2500
# Bash subprocess timeout (seconds); agent.py uses this for subprocess + bash tool description (not repeated in prompt).
ENV CMD_TIMEOUT=300
# Data-processing instructions (pre-installed list matches this image). Keep in sync with RUN apt/pip above.
ENV DATA_PROCESSING_PROMPT="Output to \`/app/output/\`.\n\
Pre-installed: python3, pip, curl, wget, jq, git, build-essential, nodejs, npm; Python: requests, httpx, pandas, numpy, matplotlib, seaborn, rich, tabulate. Install more via pip/npm/apt if needed.\n\
Minimize command output: quiet flags, redirect noise, capture only what's needed.\n\
Prevent timeouts, avoid sleeping.\n\
Phase 1: find APIs, read the docs then probe; incrementally build pipelined processing, validate end-to-end on small sample.\n\
Phase 2: run on full data."

WORKDIR /app
COPY agent.py .
# Required at run time: -e TASK_PROMPT='...' (task-specific agent instructions)
CMD ["python3", "-u", "agent.py"]
