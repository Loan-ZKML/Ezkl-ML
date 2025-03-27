#!/usr/bin/env bash
set -e

# Get the absolute path of the script's directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(dirname "$SCRIPT_DIR")"

# Check if required arguments are provided
if [ "$#" -lt 3 ]; then
    echo "Usage: $0 <input_file> <shared_circuit_dir> <output_dir> [--generate-contract]"
    echo "Example: $0 input.json shared_circuit proof_output --generate-contract"
    exit 1
fi

INPUT_FILE="$1"
SHARED_DIR="$2"
OUTPUT_DIR="$3"

# Update to absolute paths if not already absolute
if [[ "$INPUT_FILE" != /* ]]; then
    INPUT_FILE="$SCRIPT_DIR/$INPUT_FILE"
fi

if [[ "$SHARED_DIR" != /* ]]; then
    SHARED_DIR="$SCRIPT_DIR/$SHARED_DIR"
fi

if [[ "$OUTPUT_DIR" != /* ]]; then
    OUTPUT_DIR="$SCRIPT_DIR/$OUTPUT_DIR"
fi

# Update SHARED_DIR to point to the proof_generation directory
if [[ "$SHARED_DIR" == *"shared_circuit"* ]]; then
    SHARED_DIR="$SCRIPT_DIR/proof_generation"
fi

GENERATE_CONTRACT=false

# Check for optional --generate-contract flag
if [ "$#" -eq 4 ] && [ "$4" = "--generate-contract" ]; then
    GENERATE_CONTRACT=true
fi

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Check if EZKL is installed
if ! command -v ezkl &> /dev/null; then
    echo "EZKL not found. Please install it with: pip install ezkl"
    exit 1
fi

# Check if required files exist in shared directory
if [ ! -f "$SHARED_DIR/model.compiled" ] || [ ! -f "$SHARED_DIR/pk.key" ] || [ ! -f "$SHARED_DIR/vk.key" ] || [ ! -f "$SHARED_DIR/kzg.srs" ]; then
    echo "Error: Required files not found in shared directory. Please run run_ezkl_common.sh first."
    exit 1
fi

# Step 1: Generate witness with metadata flag
echo "Generating witness..."
ezkl gen-witness -D "$INPUT_FILE" -M "$SHARED_DIR/model.compiled" -O "$OUTPUT_DIR/witness.json"

# Step 2: Generate proof
echo "Generating proof..."
ezkl prove --witness "$OUTPUT_DIR/witness.json" \
         --proof-path "$OUTPUT_DIR/proof.json" \
         --pk-path "$SHARED_DIR/pk.key" \
         --compiled-circuit "$SHARED_DIR/model.compiled" \
         --srs-path "$SHARED_DIR/kzg.srs"

# Log the output data for comparison
echo "----------------------------------------"
echo "Checking for witness and metadata files:"
if [ -f "$OUTPUT_DIR/witness.json" ]; then
    echo "Witness output:"
    cat "$OUTPUT_DIR/witness.json"
fi

if [ -f "$OUTPUT_DIR/metadata.json" ]; then
    echo "Metadata output:"
    cat "$OUTPUT_DIR/metadata.json"
fi
echo "----------------------------------------"
# Copy verification key to output directory
echo "Copying verification key..."
cp "$SHARED_DIR/vk.key" "$OUTPUT_DIR/vk.key"

# After generating the proof but before verification, copy settings.json
echo "Copying settings.json..."
cp "$SHARED_DIR/settings.json" "$OUTPUT_DIR/settings.json"
# Add debug information before verification
echo "Debug: Checking file paths..."
echo "Proof path: $OUTPUT_DIR/proof.json"
echo "VK path: $OUTPUT_DIR/vk.key"
echo "SRS path: $SHARED_DIR/kzg.srs"

# Verify files exist
echo "Checking if files exist:"
[ -f "$OUTPUT_DIR/proof.json" ] && echo "✓ Proof file exists" || echo "✗ Proof file missing"
[ -f "$OUTPUT_DIR/vk.key" ] && echo "✓ VK file exists" || echo "✗ VK file missing"
[ -f "$SHARED_DIR/kzg.srs" ] && echo "✓ SRS file exists" || echo "✗ SRS file missing"

# Step 3: Verify the proof locally
echo "Running verification..."
./run_ezkl_verify.sh \
    "$OUTPUT_DIR/proof.json" \
    "$OUTPUT_DIR/vk.key" \
    "$SHARED_DIR/kzg.srs"

# Optional: Generate Solidity verifier contract and calldata
if [ "$GENERATE_CONTRACT" = true ]; then
    echo "Generating Solidity verifier contract..."
    mkdir -p "$OUTPUT_DIR/contract"
    ezkl create-evm-verifier --settings-path "$OUTPUT_DIR/settings.json" \
                           --vk-path "$OUTPUT_DIR/vk.key" \
                           --srs-path "$SHARED_DIR/kzg.srs" \
                           --sol-code-path "$OUTPUT_DIR/contract/verifier.sol"

    echo "Generating calldata for on-chain verification..."
    ezkl encode-evm-calldata --proof-path "$OUTPUT_DIR/proof.json" \
                          --calldata-path "$OUTPUT_DIR/contract/calldata.json"
fi

# Compare the synthetic score with the EZKL metadata score if available
if [ -f "$OUTPUT_DIR/metadata.json" ]; then
    echo "----------------------------------------"
    echo "EZKL Score from metadata.json:"
    if grep -q "scaled_score" "$OUTPUT_DIR/metadata.json"; then
        SCALED_SCORE=$(grep "scaled_score" "$OUTPUT_DIR/metadata.json" | sed 's/.*"scaled_score": \([0-9]*\).*/\1/')
        echo "EZKL Scaled Score: $SCALED_SCORE (0-1000 scale)"
        # Convert to 0-1 scale for comparison
        NORMALIZED_SCORE=$(echo "scale=3; $SCALED_SCORE / 1000" | bc)
        echo "EZKL Normalized Score: $NORMALIZED_SCORE (0-1 scale)"
        echo "This can be compared to the synthetic score reported earlier"
        echo "----------------------------------------"
    else
        echo "No scaled_score found in metadata.json"
    fi
fi

echo "Individual proof generation complete!"
