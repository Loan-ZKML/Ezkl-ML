fn create_ezkl_script(script_path: &Path, working_dir: &str, generate_contract: bool) -> Result<()> {
    let mut script = format!(r#"#!/usr/bin/env bash
set -e

# Change to working directory where files are located
cd "{working_dir}"

# Check if EZKL is installed
if ! command -v ezkl &> /dev/null; then
    echo "EZKL not found. Please install it with: pip install ezkl"
    exit 1
fi

# Step 1: Generate settings
echo "Generating circuit settings..."
ezkl gen-settings -M credit_model.onnx -O settings.json --logrows 20

# Update logrows in settings.json
cat settings.json | jq '.run_args.logrows = 20' > settings.tmp && mv settings.tmp settings.json
# Step 2: Calibrate settings
echo "Calibrating settings..."
ezkl calibrate-settings -M credit_model.onnx -D input.json -O settings.json --logrows 20

# Step 3: Compile model to circuit
echo "Compiling model to circuit..."
ezkl compile-circuit -M credit_model.onnx --compiled-circuit model.compiled -S settings.json --logrows 20

# Step 4: Download SRS if needed
if [ ! -f kzg.srs ]; then
    echo "Downloading SRS..."
    ezkl get-srs --srs-path kzg.srs
fi

# Step 5: Generate keys
echo "Running setup to generate keys..."
ezkl setup -M model.compiled --pk-path pk.key --vk-path vk.key --srs-path kzg.srs --logrows 20

# Step 6: Generate witness
echo "Generating witness..."
ezkl gen-witness -D input.json -M model.compiled -O witness.json

# Step 7: Generate proof
echo "Generating proof..."
ezkl prove --witness witness.json --proof-path proof.json --pk-path pk.key --compiled-circuit model.compiled --srs-path kzg.srs --logrows 20

# Step 8: Verify the proof locally
echo "Verifying proof locally..."
ezkl verify --proof-path proof.json --vk-path vk.key --srs-path kzg.srs --logrows 20

    if generate_contract {
        script.push_str(r#"

# Step 9: Generate Solidity verifier contract
echo "Generating Solidity verifier contract..."
ezkl create-evm-verifier --vk-path vk.key --sol-code-path Halo2Verifier.sol --srs-path kzg.srs

# Step 10: Generate calldata for on-chain verification
echo "Generating calldata for on-chain verification..."
ezkl encode-evm-calldata --proof-path proof.json --calldata-path calldata.json"#);
    }

    script.push_str("\n\necho \"EZKL processing complete!\"");
    fs::write(script_path, script)?;
    
    // Make the script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(script_path, perms)?;
    }
    
    Ok(())
}
