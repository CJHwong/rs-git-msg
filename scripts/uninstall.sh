#!/bin/bash

set -e

# ANSI color codes
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
NC="\033[0m" # No Color

echo -e "${YELLOW}Uninstalling rs-git-msg...${NC}"

# Define possible installation locations
POSSIBLE_LOCATIONS=(
  "$HOME/.local/bin/rs-git-msg"
  "/usr/local/bin/rs-git-msg"
  "/usr/bin/rs-git-msg"
)

# Try to find and remove the binary
FOUND=false
for location in "${POSSIBLE_LOCATIONS[@]}"; do
  if [ -f "$location" ]; then
    echo -e "${YELLOW}Found rs-git-msg at ${location}${NC}"
    if [ -w "$location" ] || [ -w "$(dirname "$location")" ]; then
      rm "$location"
      echo -e "${GREEN}Successfully removed rs-git-msg from ${location}${NC}"
      FOUND=true
    else
      echo -e "${RED}Cannot remove ${location} without sudo privileges${NC}"
      echo -e "Try running: ${YELLOW}sudo rm ${location}${NC}"
      FOUND=true
    fi
  fi
done

# Check if we found any installed binaries
if [ "$FOUND" = false ]; then
  echo -e "${RED}Could not find rs-git-msg binary in standard locations.${NC}"
  echo -e "If you installed it in a custom location, you'll need to remove it manually."
fi

# Remove any configuration files if they exist
CONFIG_DIR="$HOME/.config/rs-git-msg"
if [ -d "$CONFIG_DIR" ]; then
  rm -rf "$CONFIG_DIR"
  echo -e "${GREEN}Removed configuration directory: ${CONFIG_DIR}${NC}"
fi

echo -e "${GREEN}Uninstallation complete!${NC}"
