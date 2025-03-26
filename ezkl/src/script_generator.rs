use anyhow::{Result, Context};
use std::path::Path;
use std::process::Command;
use std::fs;

pub const MODEL_NAME: &str = "credit_model.onnx";
pub const PROOF_GEN_DIR: &str = "proof_generation";
pub const SRS_FILE: &str = "kzg.srs";

/// Creates the shared model and downloads SRS file if needed
pub fn initialize_shared_resources(features: &[f32], address: &str) -> Result<(), anyhow::Error> {
    println!("Initializing shared resources...");
    
    // Ensure proof_generation directory exists
    fs::create_dir_all(PROOF_GEN_DIR)?;

    // Generate the shared model
    let model_path = Path::new(PROOF_GEN_DIR).join(MODEL_NAME);
    if !model_path.exists() {
        println!("Generating shared model...");
        create_model(features, address, PROOF_GEN_DIR, true)?;
    } else {
        println!("Shared model already exists at {}", model_path.display());
    }

    // Get absolute paths
    let model_path_abs = fs::canonicalize(&model_path)?;
    let model_path_str = model_path_abs.to_string_lossy().into_owned();
    
    // Generate settings file
    println!("Generating settings file...");
    let settings_path = Path::new(PROOF_GEN_DIR).join("settings.json");
    let status = Command::new("ezkl")
        .arg("gen-settings")
        .arg("-M")
        .arg(&model_path_str)
        .arg("-O")
        .arg(&settings_path)
        .status()
        .context("Failed to generate settings")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to generate settings"));
    }

    // Create a dummy input.json for calibration
    create_address_input(features, address, PROOF_GEN_DIR)?;

    // Calibrate settings
    println!("Calibrating settings...");
    let status = Command::new("ezkl")
        .arg("calibrate-settings")
        .arg("-M")
        .arg(&model_path_str)
        .arg("-D")
        .arg(format!("{}/input.json", PROOF_GEN_DIR))
        .arg("-O")
        .arg(&settings_path)
        .status()
        .context("Failed to calibrate settings")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to calibrate settings"));
    }

    // Download SRS file if needed
    let srs_path = Path::new(PROOF_GEN_DIR).join(SRS_FILE);
    let settings_path = Path::new(PROOF_GEN_DIR).join("settings.json");
    if !srs_path.exists() {
        println!("Downloading SRS file...");
        let status = Command::new("ezkl")
            .arg("get-srs")
            .arg("--settings-path")  // Changed from --settings to --settings-path
            .arg(&settings_path)
            .arg("--srs-path")
            .arg(&srs_path)
            .status()
            .context("Failed to download SRS file")?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to download SRS file"));
        }
        println!("SRS file downloaded successfully");
    } else {
        println!("SRS file already exists at {}", srs_path.display());
    }

    Ok(())
}

/// Creates address-specific input.json file
pub fn create_address_input(features: &[f32], address: &str, output_dir: &str) -> Result<(), anyhow::Error> {
    println!("Creating input for address: {}", address);
    println!("Features: {:?}", features);

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    // Convert features to JSON string
    let features_json = serde_json::to_string(features)?;
    
    // Call Python script to generate input only (no model generation)
    let status = Command::new("python3")
        .arg("./script/create_model.py")
        .arg(output_dir)
        .arg(address)
        .arg(&features_json)
        .arg("0")  // Never generate model for address-specific inputs
        .status()
        .context("Failed to execute Python script")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Input creation failed with status: {}", status));
    }

    Ok(())
}

// Helper function used by initialize_shared_resources
fn create_model(features: &[f32], address: &str, output_dir: &str, force_generate_model: bool) -> Result<(), anyhow::Error> {
    // Convert features to JSON string
    let features_json = serde_json::to_string(features)?;
    
    // Call Python script to generate model
    let status = Command::new("python3")
        .arg("./script/create_model.py")
        .arg(output_dir)
        .arg(address)
        .arg(&features_json)
        .arg(if force_generate_model { "1" } else { "0" })
        .status()
        .context("Failed to execute Python script")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Model creation failed with status: {}", status));
    }

    Ok(())
}

pub fn create_ezkl_script(script_path: &Path, working_dir: &str, generate_contract: bool) -> Result<()> {
    // Ensure the shared model exists
    let model_path = Path::new(PROOF_GEN_DIR).join(MODEL_NAME);
    if !model_path.exists() {
        return Err(anyhow::anyhow!("Shared model not found at: {}", model_path.display()));
    }

    // Get absolute paths
    let working_dir_abs = fs::canonicalize(working_dir)?;
    let model_path_abs = fs::canonicalize(&model_path)?;
    let srs_path_abs = fs::canonicalize(Path::new(PROOF_GEN_DIR).join(SRS_FILE))?;

    let working_dir_str = working_dir_abs.to_string_lossy().into_owned();
    let model_path_str = model_path_abs.to_string_lossy().into_owned();
    let srs_path_str = srs_path_abs.to_string_lossy().into_owned();

    // Create a launcher bash script that calls the new Python script
    // Assumes the Python script is at /Users/mar/src/github.com/loan-zkml/ezkl/scripts/run_ezkl.py
    let mut script = format!(r#"#!/usr/bin/env bash
python3 ./script/run_ezkl.py --working-dir "{}" --model-path "{}" --srs-path "{}""#,
        working_dir_str, model_path_str, srs_path_str);
    
    if generate_contract {
        script.push_str(" --generate-contract");
    }
    
    script.push_str("\n");

    fs::write(script_path, script)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(script_path, perms)?;
    }
    
    Ok(())
}

