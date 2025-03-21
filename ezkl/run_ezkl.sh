#!/usr/bin/env bash
    set -e

    # Change to working directory where files are located
    cd "proof_generation/4444444444444444444444444444444444444444"

    # Check if EZKL is installed
    if ! command -v ezkl &> /dev/null; then
        echo "EZKL not found. Please install it with: pip install ezkl"
        exit 1
    fi

    # Step 1: Generate settings
    echo "Generating circuit settings..."
    ezkl gen-settings -M credit_model.onnx -O settings.json

    # Step 2: Calibrate settings
    echo "Calibrating settings..."
    ezkl calibrate-settings -M credit_model.onnx -D input.json -O settings.json

    # Step 3: Compile model to circuit
    echo "Compiling model to circuit..."
    ezkl compile-circuit -M credit_model.onnx --compiled-circuit model.compiled -S settings.json

    # Step 4: Download SRS if needed
    if [ ! -f kzg.srs ]; then
        echo "Downloading SRS..."
        ezkl get-srs --srs-path kzg.srs
    fi

    # Step 5: Generate keys
    echo "Running setup to generate keys..."
    ezkl setup -M model.compiled --pk-path pk.key --vk-path vk.key --srs-path kzg.srs

    # Step 6: Generate witness
    echo "Generating witness..."
    ezkl gen-witness -D input.json -M model.compiled -O witness.json

    # Step 7: Generate proof
    echo "Generating proof..."
    ezkl prove --witness witness.json --proof-path proof.json --pk-path pk.key --compiled-circuit model.compiled --srs-path kzg.srs

    # Step 8: Verify the proof locally
    echo "Verifying proof locally..."
    ezkl verify --proof-path proof.json --vk-path vk.key --srs-path kzg.srs

echo "EZKL processing complete!"