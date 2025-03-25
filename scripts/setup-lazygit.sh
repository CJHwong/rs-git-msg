#!/bin/bash

# Determine config directory based on OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_DIR="$HOME/Library/Application Support/lazygit"
else
    CONFIG_DIR="$HOME/.config/lazygit"
fi

# Create config directory path (handles paths with spaces)
mkdir -p "${CONFIG_DIR}"

# Path to config file (handles paths with spaces)
CONFIG_FILE="${CONFIG_DIR}/config.yml"

# Create config file if it doesn't exist
touch "${CONFIG_FILE}"

# Custom command configuration
read -r -d '' CONFIG << 'EOF'

customCommands:
  - key: <c-g>
    prompts:
      - type: input
        title: Additional Instructions (optional)
        key: Instructions
        initialValue: ""
      - type: menuFromCommand
        title: AI Commit Messages
        key: Msg
        command: 'rs-git-msg -n 5 {{if .Form.Instructions}}-i "{{.Form.Instructions}}"{{end}}'
    command: git commit -m "{{.Form.Msg}}"
    context: 'files'
    description: 'Generate commit message using rs-git-msg'
    loadingText: 'Generating commit messages...'
    stream: false
EOF

# Check if customCommands already exists in config
if grep -q "customCommands:" "${CONFIG_FILE}"; then
    echo "Custom commands section already exists in ${CONFIG_FILE}"
    echo "Please manually add the following configuration:"
    echo "$CONFIG"
else
    # Append the configuration
    echo "$CONFIG" >> "${CONFIG_FILE}"
    echo "Successfully added rs-git-msg command to lazygit configuration"
    echo "Use 'G' in lazygit's files view to generate a commit message"
fi

# Make script executable
chmod +x "$0"
