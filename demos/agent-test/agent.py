import anthropic, subprocess, os, sys

client = anthropic.Anthropic()
os.makedirs("/app/output/logs", exist_ok=True)

_DEFAULT_ANTHROPIC_MODEL = "claude-sonnet-4-5"
_DEFAULT_ANTHROPIC_MAX_TOKENS = 4096
_DEFAULT_CONTEXT_LIMIT = 2500


def _anthropic_model() -> str:
    # get() default only applies when the var is unset; empty/whitespace must fall back too.
    m = os.environ.get("ANTHROPIC_MODEL", _DEFAULT_ANTHROPIC_MODEL).strip()
    return m or _DEFAULT_ANTHROPIC_MODEL


def _anthropic_max_tokens() -> int:
    s = os.environ.get(
        "ANTHROPIC_MAX_TOKENS", str(_DEFAULT_ANTHROPIC_MAX_TOKENS)
    ).strip() or str(_DEFAULT_ANTHROPIC_MAX_TOKENS)
    return int(s)


def _context_limit() -> int:
    s = (
        os.environ.get("CONTEXT_LIMIT", str(_DEFAULT_CONTEXT_LIMIT)).strip()
        or str(_DEFAULT_CONTEXT_LIMIT)
    )
    return int(s)


def _cmd_timeout() -> int:
    raw = os.environ.get("CMD_TIMEOUT", "300").strip()
    try:
        t = int(raw)
    except ValueError:
        print(f"ERROR: CMD_TIMEOUT must be an integer, got {raw!r}", file=sys.stderr)
        sys.exit(1)
    if t < 1:
        print("ERROR: CMD_TIMEOUT must be >= 1", file=sys.stderr)
        sys.exit(1)
    return t


CMD_TIMEOUT = _cmd_timeout()
ANTHROPIC_MODEL = _anthropic_model()
ANTHROPIC_MAX_TOKENS = _anthropic_max_tokens()
CONTEXT_LIMIT = _context_limit()


def _load_data_processing_prompt() -> str:
    prompt = os.environ.get("DATA_PROCESSING_PROMPT", "").strip()
    if not prompt:
        print(
            "WARNING: DATA_PROCESSING_PROMPT is unset; continuing without data-processing instructions.",
            file=sys.stderr,
        )
    return prompt


def _load_task_prompt() -> str:
    task = os.environ.get("TASK_PROMPT", "").strip()
    if not task:
        print(
            "ERROR: TASK_PROMPT environment variable is required (task instructions for the agent).",
            file=sys.stderr,
        )
        sys.exit(1)
    return task


DATA_PROCESSING_PROMPT = _load_data_processing_prompt()
TASK_PROMPT = _load_task_prompt()
PROMPT = TASK_PROMPT + "\n\n" + DATA_PROCESSING_PROMPT

tools = [
    {
        "name": "bash",
        "description": f"Run a shell command (hard timeout {CMD_TIMEOUT}s)",
        "input_schema": {
            "type": "object",
            "properties": {"command": {"type": "string"}},
            "required": ["command"]
        }
    },
    {
        "type": "web_search_20250305",
        "name": "web_search"
    }
]

messages = [{"role": "user", "content": PROMPT}]

while True:
    response = client.messages.create(
        model=ANTHROPIC_MODEL,
        max_tokens=ANTHROPIC_MAX_TOKENS,
        tools=tools,
        messages=messages
    )
    messages.append({"role": "assistant", "content": response.content})

    for block in response.content:
        if hasattr(block, "text"):
            print(block.text, flush=True)
        elif block.type == "tool_use" and block.name == "bash":
            cmd = block.input.get("command", "") if isinstance(block.input, dict) else ""
            print(f"\n[tool] {cmd}", flush=True)

    if response.stop_reason == "end_turn":
        break

    tool_results = []
    for block in response.content:
        if block.type != "tool_use" or block.name != "bash":
            continue

        cmd = block.input.get("command", "") if isinstance(block.input, dict) else ""
        if not cmd:
            tool_results.append({
                "type": "tool_result",
                "tool_use_id": block.id,
                "content": "ERROR: no or malformed command in tool input"
            })
            continue

        result = None
        try:
            result = subprocess.run(
                cmd, shell=True,
                capture_output=True, text=True,
                timeout=CMD_TIMEOUT
            )
            output = result.stdout + result.stderr
        except subprocess.TimeoutExpired:
            output = f"ERROR: command timed out after {CMD_TIMEOUT}s: {cmd}"

        # Save full output to log
        log_path = f"/app/output/logs/cmd_{block.id}.log"
        with open(log_path, "w") as f:
            f.write(f"$ {cmd}\n\n{output}")

        # Truncate long output; full log always in log_path
        suffix = f"\n[... truncated, full output in {log_path}]"
        content = output if len(output) <= CONTEXT_LIMIT else output[: CONTEXT_LIMIT - len(suffix)] + suffix

        print(f"[result] {output[:200]}", flush=True)
        tool_results.append({
            "type": "tool_result",
            "tool_use_id": block.id,
            "content": content
        })

    messages.append({"role": "user", "content": tool_results})
