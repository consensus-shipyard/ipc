# agent-test

Dockerized Anthropic agent loop: **Claude** with **bash** (timed) and **web_search**, running a task you supply. It writes artifacts under `output/` (e.g. `report.md`, scripts, command logs).

## Prerequisites

- Docker
- `ANTHROPIC_API_KEY` in your environment

## Usage

1. Edit **`task.txt`** with your task instructions (what to analyze, deliverables, where to write results).
2. From this directory:

   ```bash
   export ANTHROPIC_API_KEY=sk-ant-...
   ./run.sh
   ```

   This builds image `ipc-agent-test`, clears/recreates **`output/`**, mounts it at `/app/output` in the container, and runs the agent as your user so files are easy to delete locally.

3. Inspect results in **`output/`**.

## Customizing

- **Task only:** change `task.txt` (or pass `-e TASK_PROMPT='...'` if you run `docker` yourself).
- **Image defaults:** `Dockerfile` sets `DATA_PROCESSING_PROMPT` (how to work / what’s pre-installed) and `CMD_TIMEOUT` for bash. Rebuild after changing them.

## Configuration (environment)

| Variable | Role | Default (image / agent) |
|----------|------|-------------------------|
| `ANTHROPIC_MODEL` | Claude model id for `messages.create` | `claude-sonnet-4-5` |
| `ANTHROPIC_MAX_TOKENS` | Max output tokens per turn | `4096` |
| `CONTEXT_LIMIT` | Max chars of each bash result returned to the model | `2500` |
| `CMD_TIMEOUT` | Bash subprocess timeout (seconds) | `300` |
| `TASK_PROMPT` | Task instructions | required at run time (`run.sh` uses `task.txt`) |
| `DATA_PROCESSING_PROMPT` | Generic workflow / pre-install hints | set in `Dockerfile` |

**Examples**

```bash
# Different model for one run (with ./run.sh — export before invoking)
export ANTHROPIC_MODEL=claude-opus-4-5
./run.sh
```

```bash
# docker run: override model / max tokens
docker run --rm --user "$(id -u):$(id -g)" \
  -e ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY" \
  -e ANTHROPIC_MODEL=claude-opus-4-5 \
  -e ANTHROPIC_MAX_TOKENS=8192 \
  -e TASK_PROMPT="$(cat task.txt)" \
  -v "$(pwd)/output:/app/output" \
  ipc-agent-test
```

## Manual run (optional)

```bash
docker build -t ipc-agent-test .
docker run --rm --user "$(id -u):$(id -g)" \
  -e ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY" \
  -e TASK_PROMPT="$(cat task.txt)" \
  -v "$(pwd)/output:/app/output" \
  ipc-agent-test
```
