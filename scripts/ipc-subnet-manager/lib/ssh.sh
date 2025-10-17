#!/bin/bash
# SSH helper functions

# Execute command on remote host as IPC user
ssh_exec() {
    local ip="$1"
    local ssh_user="$2"
    local ipc_user="$3"
    shift 3
    local cmd="$*"

    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] Would execute on $ip: $cmd"
        return 0
    fi

    ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 "$ssh_user@$ip" \
        "sudo su - $ipc_user -c '$cmd'" 2>&1
}

# Execute command without sudo/su wrapping (for direct execution)
ssh_exec_direct() {
    local ip="$1"
    local ssh_user="$2"
    local ipc_user="$3"
    shift 3
    local cmd="$*"

    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] Would execute on $ip: $cmd"
        return 0
    fi

    ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 "$ssh_user@$ip" \
        "sudo su - $ipc_user -c 'bash -l -c \"$cmd\"'"
}

# Test SSH connectivity
test_ssh() {
    local ip="$1"
    local ssh_user="$2"

    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] Would test SSH to $ssh_user@$ip"
        return 0  # Always succeed in dry-run
    fi

    ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 -o BatchMode=yes \
        "$ssh_user@$ip" "exit" >/dev/null 2>&1
}

# Copy file to remote host
scp_to_host() {
    local ip="$1"
    local ssh_user="$2"
    local ipc_user="$3"
    local local_file="$4"
    local remote_path="$5"

    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] Would copy $local_file to $ip:$remote_path"
        return 0
    fi

    # Copy to temp location
    local temp_file="/tmp/$(basename "$local_file")"
    scp -o StrictHostKeyChecking=no "$local_file" "$ssh_user@$ip:$temp_file" >/dev/null 2>&1

    # Move to final location with correct ownership
    ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo mv $temp_file $remote_path && sudo chown $ipc_user:$ipc_user $remote_path"
}

# Get file from remote host
scp_from_host() {
    local ip="$1"
    local ssh_user="$2"
    local ipc_user="$3"
    local remote_path="$4"
    local local_file="$5"

    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] Would copy $ip:$remote_path to $local_file"
        return 0
    fi

    # Copy to temp location first
    local temp_file="/tmp/$(basename "$remote_path")"
    ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo cp $remote_path $temp_file && sudo chown $ssh_user:$ssh_user $temp_file"

    scp -o StrictHostKeyChecking=no "$ssh_user@$ip:$temp_file" "$local_file" >/dev/null 2>&1

    # Cleanup
    ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" "rm -f $temp_file"
}

# Check if process is running on remote host
ssh_check_process() {
    local ip="$1"
    local ssh_user="$2"
    local ipc_user="$3"
    local process_name="$4"

    ssh_exec "$ip" "$ssh_user" "$ipc_user" "pgrep -f '$process_name' >/dev/null && echo 'running' || echo 'stopped'"
}

# Kill process on remote host
ssh_kill_process() {
    local ip="$1"
    local ssh_user="$2"
    local ipc_user="$3"
    local process_name="$4"

    ssh_exec "$ip" "$ssh_user" "$ipc_user" "pkill -f '$process_name' || true"
}

