#!/usr/bin/env bash
set -e

# Default values
SHARED_DIR="shared_circuit"
MODEL_PATH=""
SRS_PATH=""
GENERATE_CONTRACT=false

# Display usage information
show_usage() {
    echo "Usage: $0 [--setup-common --model-path <path> --srs-path <path>] [--generate-contract] <address_dir>"
    echo
    echo "Options:"
    echo "  --setup-common       Generate common circuit and keys (only needed once)"
    echo "  --model-path <path>  Path to the ONNX model file (required with --setup-common)"
    echo "  --srs-path <path>    Path to the SRS file (required with --setup-common)"
    echo "  --generate-contract  Generate Solidity verifier contract and calldata"
    echo "  <address_dir>        Directory containing address-specific input.json file"
    echo
    echo "Examples:"
    echo "  $0 --setup-common --model-path model.onnx --srs-path kzg.srs"
    echo "  $0 proof_generation/4444444444444444444444444444444444444444"
    echo "  $0 --generate-contract proof_generation/4444444444444444444444444444444444444444"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --setup-common)
            SETUP_COMMON=true
            shift
            ;;
        --model-path)
            MODEL_PATH="$2"
            shift 2
            ;;
        --srs-path)
            SRS_PATH="$2"
            shift 2
            ;;
        --generate-contract)
            GENERATE_CONTRACT=true
            shift
            ;;
        -h|--help)
            show_usage
            ;;
        *)
            if [ -z "$ADDRESS_DIR" ]; then
                ADDRESS_DIR="$1"
            else
                echo "Error: Unexpected argument '$1'"
                show_usage
            fi
            shift
            ;;
    esac
done

# Validate arguments
if [ "$SETUP_COMMON" = true ]; then
    if [ -z "$MODEL_PATH" ] || [ -z "$SRS_PATH" ]; then
        echo "Error: --model-path and --srs-path are required with --setup-common"
        show_usage
    fi
    
    echo "Generating common circuit and keys..."
    ./run_ezkl_common.sh "$MODEL_PATH" "$SHARED_DIR" "$SRS_PATH"
    exit 0
fi

if [ -z "$ADDRESS_DIR" ]; then
    echo "Error: Address directory is required"
    show_usage
fi

# Validate address directory
if [ ! -d "$ADDRESS_DIR" ]; then
    echo "Error: Address directory '$ADDRESS_DIR' not found"
    exit 1
fi

if [ ! -f "$ADDRESS_DIR/input.json" ]; then
    echo "Error: input.json not found in '$ADDRESS_DIR'"
    exit 1
fi

# Generate proof for the specific address
echo "Generating proof for address in $ADDRESS_DIR..."
if [ "$GENERATE_CONTRACT" = true ]; then
    ./run_ezkl_individual.sh "$ADDRESS_DIR/input.json" "$SHARED_DIR" "$ADDRESS_DIR" --generate-contract
else
    ./run_ezkl_individual.sh "$ADDRESS_DIR/input.json" "$SHARED_DIR" "$ADDRESS_DIR"
fi
