#!/usr/bin/env bash
set -e

# Get the absolute path of the script's directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if required arguments are provided
if [ "$#" -lt 3 ]; then
    echo "Usage: $0 <model_path> <output_dir> <srs_path>"
    exit 1
fi

MODEL_PATH="$1"
OUTPUT_DIR="$2"
SRS_PATH="$3"

# Convert to absolute paths if needed
if [[ "$MODEL_PATH" != /* ]]; then
    MODEL_PATH="$SCRIPT_DIR/$MODEL_PATH"
fi

if [[ "$OUTPUT_DIR" != /* ]]; then
    OUTPUT_DIR="$SCRIPT_DIR/$OUTPUT_DIR"
fi

if [[ "$SRS_PATH" != /* ]]; then
    SRS_PATH="$SCRIPT_DIR/$SRS_PATH"
fi

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Step 1: Compile the circuit
echo "Compiling circuit..."
ezkl compile-circuit --model "$MODEL_PATH" \
                   --compiled-circuit "$OUTPUT_DIR/model.compiled" \
                   --settings-path "$OUTPUT_DIR/settings.json"

# Step 2: Generate circuits and keys
echo "Generating circuits and keys..."
ezkl setup --compiled-circuit "$OUTPUT_DIR/model.compiled" \
         --srs-path "$SRS_PATH" \
         --vk-path "$OUTPUT_DIR/vk.key" \
         --pk-path "$OUTPUT_DIR/pk.key"

echo "Circuit and key generation completed successfully"
