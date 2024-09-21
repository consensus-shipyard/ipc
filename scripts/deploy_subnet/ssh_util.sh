#!/bin/bash

set -euo pipefail

# Define the SSH key locations
declare -A REPO_KEYS=(
  ["ipc"]="$HOME/.ssh/id_ed25519.hokunet.ipc"
  ["builtin-actors"]="$HOME/.ssh/id_ed25519.hokunet.builtin-actors"
  ["contracts"]="$HOME/.ssh/id_ed25519.hokunet.contracts"
)

# Define the SSH alias to submodule mapping
declare -A SUBMODULE_URLS=(
  ["builtin-actors"]="git@github.com-builtin-actors:hokunet/builtin-actors.git"
  ["hoku-contracts"]="git@github.com-contracts:hokunet/contracts.git"
)

# Add SSH aliases to ~/.ssh/config
setup_ssh_config() {
    echo "Setting up SSH config..."
    for repo in "${!REPO_KEYS[@]}"; do
        # Check if the alias is already present in the config
        if ! grep -q "Host github.com-$repo" ~/.ssh/config; then
            echo "Adding SSH alias for $repo"
            cat >> ~/.ssh/config <<EOF

Host github.com-$repo
    HostName github.com
    User git
    IdentityFile ${REPO_KEYS[$repo]}
    IdentitiesOnly yes
EOF
        fi
    done
}

# Update .gitmodules file
update_gitmodules() {
    echo "Updating .gitmodules for private submodules..."
    for submodule in "${!SUBMODULE_URLS[@]}"; do
        # Modify the .gitmodules URL for the submodule
        git config -f .gitmodules submodule."$submodule".url "${SUBMODULE_URLS[$submodule]}"
    done
}

# Revert changes to .gitmodules file
revert_gitmodules() {
    echo "Reverting changes in .gitmodules..."
    git checkout .gitmodules
}

# Add SSH keys to the ssh-agent
add_ssh_keys() {
    eval "$(ssh-agent -s)"
    for key in "${REPO_KEYS[@]}"; do
        ssh-add "$key"
    done
}
