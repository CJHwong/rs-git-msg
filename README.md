# rs-git-msg

An AI-powered git commit message generator written in Rust.

## Features

- Automatically generates commit messages based on staged changes in your repository
- Follows the [Conventional Commits](https://www.conventionalcommits.org/) format
- Supports multiple AI providers:
  - [Ollama](https://ollama.ai/) (local inference)
  - OpenAI API (GPT models)
- Customizable with different models and parameters
- Generate multiple message options

## Installation

### Using the install script (recommended)

The easiest way to install rs-git-msg is using the provided install script:

```bash
# Clone the repository
git clone https://github.com/yourusername/rs-git-msg.git
cd rs-git-msg

# Run the install script
./scripts/install.sh
```

The script will:

- Build the binary with optimizations
- Install it to an appropriate location in your PATH
- Set up necessary environment configurations if needed

### From source (manual)

1. Clone the repository
2. Build with Cargo:

```bash
cargo build --release
```

3. Move the built executable to a location in your PATH:

```bash
cp target/release/rs-git-msg ~/.local/bin/
# or
sudo cp target/release/rs-git-msg /usr/local/bin/
```

## Uninstallation

### Using the uninstall script

To remove rs-git-msg from your system, you can use the uninstall script:

```bash
./scripts/uninstall.sh
```

This script will:
- Remove the rs-git-msg binary from standard installation locations
- Clean up any configuration files created during use

### Manual uninstallation

To manually uninstall, simply remove the binary from where you installed it:

```bash
# If installed to ~/.local/bin
rm ~/.local/bin/rs-git-msg

# Or if installed to /usr/local/bin
sudo rm /usr/local/bin/rs-git-msg

# Optionally remove config files
rm -rf ~/.config/rs-git-msg
```

## Usage

Basic usage:

```bash
# Stage some changes first
git add .

# Generate a commit message
rs-git-msg
```

### Options

```
Usage: rs-git-msg [OPTIONS]

Options:
  -n, --number <NUMBERS>    Number of commit messages to generate (1-5) [default: 1]
  -i, --instructions <INSTRUCTIONS>
                            Additional context or instructions for the AI
  -v, --verbose             Enable verbose output
  -p, --provider <PROVIDER> AI provider to use [default: ollama] [possible values: ollama, openai]
  -m, --model <MODEL>       Model name to use [default: qwen2.5-coder]
  -k, --api-key <API_KEY>   API key for the provider (not needed for Ollama)
  -u, --api-url <API_URL>   API base URL (defaults to provider's standard URL)
  -h, --help                Print help
  -V, --version             Print version
```

### Examples

```bash
# Using Ollama with a different model
rs-git-msg -m llama3

# Generate 3 message options
rs-git-msg -n 3

# Using OpenAI's GPT-3.5 Turbo
rs-git-msg -p openai -m gpt-3.5-turbo -k your_api_key_here

# Enable verbose output for debugging
rs-git-msg -v
```

## Environment Variables

- `RS_GIT_MSG_API_KEY`: Set your API key for OpenAI

## AI Provider Setup

### Ollama (Default)

1. [Install Ollama](https://ollama.ai/download)
2. Pull the desired model: `ollama pull qwen2.5-coder` (or another model of your choice)
3. Run rs-git-msg (no API key needed)

### OpenAI

1. Create an account at [OpenAI](https://platform.openai.com/)
2. Generate an API key
3. Run rs-git-msg with `-p openai -k your_api_key`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
