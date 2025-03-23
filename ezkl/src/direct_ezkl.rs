use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::onnx_converter::convert_json_to_onnx;

/// This module provides source code EZKL integration taking advantage of EZKL Rust implementation.
/// When using the source code integration mode, the execution does not rely on a shell script
/// It's used as an alternative to the shell script.
#[allow(dead_code)]
pub fn run_ezkl_process(working_dir: &Path) -> Result<()> {
    println!("Running EZKL processing directly...");
    println!("Working directory: {}", working_dir.display());

    // Define paths for EZKL artifacts
    let json_model_path = working_dir.join("credit_model.json");
    let json_input_path = working_dir.join("input.json");
    let onnx_model_path = working_dir.join("credit_model.onnx");
    let ezkl_input_path = working_dir.join("ezkl_input.json");
    let settings_path = working_dir.join("settings.json");
    let compiled_path = working_dir.join("model.compiled");
    let srs_path = working_dir.join("kzg.srs");
    let vk_path = working_dir.join("vk.key");
    let pk_path = working_dir.join("pk.key");
    let witness_path = working_dir.join("witness.json");
    let proof_path = working_dir.join("proof.json");
    let verifier_path = working_dir.join("Halo2Verifier.sol");
    let calldata_path = working_dir.join("calldata.json");

    // Check if required input files exist
    if !json_model_path.exists() {
        return Err(anyhow::anyhow!(
            "Required file not found: {}",
            json_model_path.display()
        ));
    }

    if !json_input_path.exists() {
        return Err(anyhow::anyhow!(
            "Required file not found: {}",
            json_input_path.display()
        ));
    }

    // First step: Convert JSON model to ONNX format
    println!("Converting JSON model to ONNX format...");
    convert_json_to_onnx(
        &json_model_path,
        &json_input_path,
        &onnx_model_path,
        &ezkl_input_path,
    )?;

    // Step 1: Generate settings
    println!("Generating circuit settings...");
    execute_ezkl(
        &[
            "gen-settings",
            "-M",
            &onnx_model_path.to_string_lossy(),
            "-O",
            &settings_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    // Step 2: Calibrate settings
    println!("Calibrating settings...");
    execute_ezkl(
        &[
            "calibrate-settings",
            "-M",
            &onnx_model_path.to_string_lossy(),
            "-D",
            &ezkl_input_path.to_string_lossy(),
            "-O",
            &settings_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    // Step 3: Compile model to circuit
    println!("Compiling model to circuit...");
    execute_ezkl(
        &[
            "compile-circuit",
            "-M",
            &onnx_model_path.to_string_lossy(),
            "--compiled-circuit",
            &compiled_path.to_string_lossy(),
            "-S",
            &settings_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    // Step 4: Download SRS if it doesn't exist
    if !srs_path.exists() {
        println!("Downloading SRS...");
        execute_ezkl(
            &["get-srs", "--srs-path", &srs_path.to_string_lossy()],
            working_dir,
        )?;
    }

    // Step 5: Setup (generate keys)
    println!("Running setup to generate keys...");
    execute_ezkl(
        &[
            "setup",
            "-M",
            &compiled_path.to_string_lossy(),
            "--pk-path",
            &pk_path.to_string_lossy(),
            "--vk-path",
            &vk_path.to_string_lossy(),
            "--srs-path",
            &srs_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    // Step 6: Generate witness
    println!("Generating witness...");
    execute_ezkl(
        &[
            "gen-witness",
            "-D",
            &ezkl_input_path.to_string_lossy(),
            "-M",
            &compiled_path.to_string_lossy(),
            "-O",
            &witness_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    // Step 7: Generate proof
    println!("Generating proof...");
    execute_ezkl(
        &[
            "prove",
            "--witness",
            &witness_path.to_string_lossy(),
            "--proof-path",
            &proof_path.to_string_lossy(),
            "--pk-path",
            &pk_path.to_string_lossy(),
            "--compiled-circuit",
            &compiled_path.to_string_lossy(),
            "--srs-path",
            &srs_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    // Step 8: Verify proof
    println!("Verifying proof...");
    execute_ezkl(
        &[
            "verify",
            "--proof-path",
            &proof_path.to_string_lossy(),
            "--vk-path",
            &vk_path.to_string_lossy(),
            "--srs-path",
            &srs_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    // Step 9: Generate EVM verifier
    println!("Generating Solidity verifier contract...");
    execute_ezkl(
        &[
            "create-evm-verifier",
            "--vk-path",
            &vk_path.to_string_lossy(),
            "--sol-code-path",
            &verifier_path.to_string_lossy(),
            "--srs-path",
            &srs_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    // Step 10: Generate calldata
    println!("Generating calldata for on-chain verification...");
    execute_ezkl(
        &[
            "encode-evm-calldata",
            "--proof-path",
            &proof_path.to_string_lossy(),
            "--calldata-path",
            &calldata_path.to_string_lossy(),
        ],
        working_dir,
    )?;

    println!("ZKML credit scoring system setup complete!");
    println!("Generated artifacts:");
    println!(" - Model: {}", onnx_model_path.display());
    println!(" - Settings: {}", settings_path.display());
    println!(" - Circuit: {}", compiled_path.display());
    println!(" - Verification key: {}", vk_path.display());
    println!(" - Proving key: {}", pk_path.display());
    println!(" - Witness: {}", witness_path.display());
    println!(" - Proof: {}", proof_path.display());
    println!(" - EVM verifier contract: {}", verifier_path.display());
    println!(" - On-chain calldata: {}", calldata_path.display());

    Ok(())
}

/// Helper function to execute EZKL commands
#[allow(dead_code)]
fn execute_ezkl(args: &[&str], working_dir: &Path) -> Result<()> {
    // For debugging - print command
    let cmd_str = format!("ezkl {}", args.join(" "));
    println!("Executing: {}", cmd_str);

    let status = Command::new("ezkl")
        .args(args)
        .current_dir(working_dir)
        .status()
        .context(format!("Failed to execute command: {}", cmd_str))?;

    if !status.success() {
        return Err(anyhow::anyhow!("Command failed with status: {}", status));
    }

    Ok(())
}
