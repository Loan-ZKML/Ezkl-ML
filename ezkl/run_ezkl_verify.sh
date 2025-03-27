#!/usr/bin/env bash
set -e

# Get the absolute path of the script's directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if required arguments are provided
if [ "$#" -lt 3 ]; then
    echo "Usage: $0 <proof_path> <vk_path> <srs_path>"
    exit 1
fi

PROOF_PATH="$1"
VK_PATH="$2"
SRS_PATH="$3"

# Get the directory containing the proof
PROOF_DIR=$(dirname "$PROOF_PATH")

# Check for settings.json in the proof directory
if [ ! -f "$PROOF_DIR/settings.json" ]; then
    echo "Error: settings.json not found in $PROOF_DIR"
    exit 1
fi

# Convert to absolute paths if needed
if [[ "$PROOF_PATH" != /* ]]; then
    PROOF_PATH="$SCRIPT_DIR/$PROOF_PATH"
fi

if [[ "$VK_PATH" != /* ]]; then
    VK_PATH="$SCRIPT_DIR/$VK_PATH"
fi

if [[ "$SRS_PATH" != /* ]]; then
    SRS_PATH="$SCRIPT_DIR/$SRS_PATH"
fi

echo "Verifying proof..."
ezkl verify --proof-path "$PROOF_PATH" \
          --vk-path "$VK_PATH" \
          --srs-path "$SRS_PATH" \
          --settings-path "$PROOF_DIR/settings.json"

echo "Proof verification completed successfully"
