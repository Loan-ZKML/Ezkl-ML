mod direct_ezkl;
mod onnx_converter;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::process::Command;

const CONTRACTS_SRC_PATH: &str = "../../contracts/src";
const CONTRACTS_SCRIPT_PATH: &str = "../../contracts/script";

// Define structures for metadata
#[derive(Serialize, Deserialize)]
struct ProofMetadata {
    proof_hash: String,
    credit_score: u32,
    timestamp: u64,
    model_version: String,
}

fn main() -> Result<()> {
    // Generate feature data for model
    let features = vec![0.8, 0.7, 0.6, 1.0]; // [tx_count, wallet_age, avg_balance, repayment_history]

    // Create directory for artifacts
    fs::create_dir_all("proof_generation")?;
    fs::create_dir_all("script")?;

    // Step 1: Create model and input
    create_model_script(&features)?;

    // Step 2: Run Python script to create ONNX model and input
    println!("Creating model and preparing input...");
    let status = Command::new("python3")
        .arg("create_model.py")
        .status()
        .context("Failed to execute Python script")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Model creation script failed with status: {}",
            status
        ));
    }

    // Step 3: Generate proof with EZKL
    println!("Processing with EZKL...");
    let script_path = Path::new("run_ezkl.sh");
    create_ezkl_script(script_path)?;

    // Make executable
    Command::new("chmod")
        .arg("+x")
        .arg(script_path)
        .status()
        .context("Failed to make script executable")?;

    // Run EZKL script
    let status = Command::new("bash")
        .arg(script_path)
        .status()
        .context("Failed to execute EZKL script")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "EZKL script failed with status: {}",
            status
        ));
    }

    // Step 4: Create proof registry for tracking
    println!("Creating proof registry...");
    create_proof_registry()?;

    // Step 5: Copy artifacts to appropriate locations
    println!("Copying artifacts for Solidity tests...");

    // Copy files
    fs::create_dir_all(CONTRACTS_SRC_PATH)?;
    fs::create_dir_all(CONTRACTS_SCRIPT_PATH)?;
    fs::copy(
        "proof_generation/Halo2Verifier.sol",
        format!("{}/Halo2Verifier.sol", CONTRACTS_SRC_PATH),
    )?;
    fs::copy(
        "proof_generation/calldata.json",
        format!("{}/calldata.json", CONTRACTS_SCRIPT_PATH),
    )?;

    println!("Proof generation complete!");
    println!("Generated artifacts:");
    println!(" - Model: proof_generation/credit_model.onnx");
    println!(" - Verification key: proof_generation/vk.key");
    println!(" - Proof: proof_generation/proof.json");
    println!(" - Verifier contract: proof_generation/Halo2Verifier.sol");
    println!(" - On-chain calldata: proof_generation/calldata.json");

    Ok(())
}

fn create_model_script(features: &[f32]) -> Result<()> {
    let script = format!(
        r#"
import json
import numpy as np
import torch
import torch.nn as nn
from torch.onnx import export
import time

# Define a simple model - linear with sigmoid
class SimpleModel(nn.Module):
    def __init__(self):
        super(SimpleModel, self).__init__()
        # Simple linear model with sigmoid activation
        self.linear = nn.Linear(4, 1)

        # Set weights directly - this is a simplified version of the model
        # We use weights that match a simple credit scoring formula
        weight = torch.tensor([[0.3, 0.2, 0.2, 0.3]]).float()
        self.linear.weight.data = weight
        self.linear.bias.data = torch.tensor([0.0])

        # Sigmoid to normalize output between 0 and 1
        self.sigmoid = nn.Sigmoid()

    def forward(self, x):
        x = self.linear(x)
        x = self.sigmoid(x)
        return x

# Create the model
model = SimpleModel()
model.eval()

# Define features for credit score calculation
favorable_features = {:?}  # [tx_count, wallet_age, avg_balance, repayment_history]

# Calculate score
input_tensor = torch.tensor([favorable_features], dtype=torch.float32)
with torch.no_grad():
    score = model(input_tensor).item()

print(f"Features: {{favorable_features}}")
print(f"Calculated score: {{score:.4f}}")
print(f"Threshold for favorable rate: 0.5")
print(f"Qualifies for favorable rate (100% collateral): {{score > 0.5}}")

# Export to ONNX
export(
    model,
    input_tensor,
    "proof_generation/credit_model.onnx",
    input_names=["input"],
    output_names=["output"],
    dynamic_axes={{"input": {{0: "batch_size"}}, "output": {{0: "batch_size"}}}}
)

# Create EZKL input
# Scale the score to be between 0-1000 for easier integer comparison in smart contracts
scaled_score = int(score * 1000)
print(f"Scaled score (0-1000): {{scaled_score}}")

ezkl_input = {{
  "input_shapes": [[4]],
  "input_data": [favorable_features],
  # This is important - we include the scaled score as a public output
  # This will become a public input to the verification system
  "output_data": [[scaled_score / 1000.0]]
}}

with open("proof_generation/input.json", "w") as f:
    json.dump(ezkl_input, f, indent=2)

# Also save metadata for later reference
metadata = {{
    "features": favorable_features,
    "score": score,
    "scaled_score": scaled_score,
    "timestamp": int(time.time()),
    "model_version": "1.0.0"
}}

with open("proof_generation/metadata.json", "w") as f:
    json.dump(metadata, f, indent=2)

print("Model converted to ONNX and input prepared for EZKL")
"#,
        features
    );

    fs::write("create_model.py", script)?;
    Ok(())
}

fn create_ezkl_script(path: &Path) -> Result<()> {
    let script = r#"#!/usr/bin/env bash
set -e

cd proof_generation

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

# Step 9: Generate Solidity verifier contract
echo "Generating Solidity verifier contract..."
ezkl create-evm-verifier --vk-path vk.key --sol-code-path Halo2Verifier.sol --srs-path kzg.srs

# Step 10: Generate calldata for on-chain verification
echo "Generating calldata for on-chain verification..."
ezkl encode-evm-calldata --proof-path proof.json --calldata-path calldata.json

echo "EZKL processing complete!"
"#;

    fs::write(path, script)?;
    Ok(())
}

fn create_proof_registry() -> Result<()> {
    // Create a proof registry to track proofs
    fs::create_dir_all("proof_registry")?;

    // Calculate proof hash
    let proof_data = fs::read("proof_generation/proof.json")?;
    let mut hasher = Sha256::new();
    hasher.update(&proof_data);
    let result = hasher.finalize();
    let proof_hash = hex::encode(result);

    // Extract credit score from witness.json
    let witness_data = fs::read_to_string("proof_generation/witness.json")?;
    let witness: serde_json::Value = serde_json::from_str(&witness_data)?;

    // Get the credit score from the output data
    // First try to get it from the hex representation in the outputs
    const SCORE_SCALER: f64 = 1000.0;
    let scaled_score = if let Some(output_hex) = witness["outputs"][0][0].as_str() {
        // Convert from hex to u32
        // The output is a hex string like "1416000000000000000000000000000000000000000000000000000000000000"
        // We need to take the first 4 characters (after 0x) which represent our score
        let score_hex = &output_hex[0..4]; // Take first 4 characters
        u32::from_str_radix(score_hex, 16).unwrap_or(0)
    } else if let Some(rescaled_output) =
        witness["pretty_elements"]["rescaled_outputs"][0][0].as_str()
    {
        // If the pretty_elements path exists and contains a string, parse it
        let float_val = rescaled_output.parse::<f64>().unwrap_or(0.0);
        (float_val * SCORE_SCALER).round() as u32
    } else if let Some(float_val) = witness["pretty_elements"]["rescaled_outputs"][0][0].as_f64() {
        // If it's directly a number value
        (float_val * SCORE_SCALER).round() as u32
    } else {
        // Fallback - use a default value if we can't extract it
        println!("Warning: Could not extract credit score from witness. Using default value 500.");
        500
    };

    println!("Extracted credit score: {}", scaled_score);

    // Create registry entry
    let registry_entry = ProofMetadata {
        proof_hash,
        credit_score: scaled_score,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        model_version: "1.0.0".to_string(),
    };

    // Save registry entry
    let registry_path = format!("proof_registry/{}.json", registry_entry.proof_hash);
    fs::write(
        &registry_path,
        serde_json::to_string_pretty(&registry_entry)?,
    )?;

    // Create lookup file for testing
    let lookup = serde_json::json!({
        "proof_hash": registry_entry.proof_hash,
        "credit_score": registry_entry.credit_score,
        "public_input": format!("0x{:x}", registry_entry.credit_score),
    });

    fs::write(
        "script/proof_lookup.json",
        serde_json::to_string_pretty(&lookup)?,
    )?;

    println!(
        "Created proof registry with hash: {}",
        registry_entry.proof_hash
    );

    Ok(())
}
