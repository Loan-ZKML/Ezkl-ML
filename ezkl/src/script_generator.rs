use anyhow::{Result, Context, anyhow};
use std::path::Path;
use std::process::Command;
use std::fs;
use colored::*;

pub const MODEL_NAME: &str = "credit_model.onnx";
pub const PROOF_GEN_DIR: &str = "proof_generation";
pub const SRS_FILE: &str = "kzg.srs";

// Define shell script paths
pub const SHELL_SCRIPTS: &[&str] = &[
    "./run_ezkl.sh",
    "./run_ezkl_common.sh",
    "./run_ezkl_individual.sh"
];

/// Log a status message with timestamp
fn log_status(message: &str) {
    println!("[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message);
}

/// Log a success message
fn log_success(message: &str) {
    println!("[SUCCESS] {}", message.green());
}

/// Log an error message
fn log_error(message: &str) {
    eprintln!("[ERROR] {}", message.red());
}

/// Log a warning message
#[allow(dead_code)]
fn log_warning(message: &str) {
    println!("[WARNING] {}", message.yellow());
}

/// Log an info message
fn log_info(message: &str) {
    println!("[INFO] {}", message.blue());
}

/// Creates the shared model and downloads SRS file if needed
pub fn initialize_shared_resources(features: &[f32], address: &str) -> Result<(), anyhow::Error> {
    log_status("Initializing shared resources...");
    
    // Ensure proof_generation directory exists
    match fs::create_dir_all(PROOF_GEN_DIR) {
        Ok(_) => log_info(&format!("Directory '{}' is ready", PROOF_GEN_DIR)),
        Err(e) => {
            log_error(&format!("Failed to create directory '{}': {}", PROOF_GEN_DIR, e));
            return Err(anyhow::anyhow!("Failed to create directory '{}': {}", PROOF_GEN_DIR, e));
        }
    };

    // Generate the shared model
    let model_path = Path::new(PROOF_GEN_DIR).join(MODEL_NAME);
    if !model_path.exists() {
        log_status("Generating shared model...");
        match create_model(features, address, PROOF_GEN_DIR, true) {
            Ok(_) => log_success("Shared model created successfully"),
            Err(e) => {
                log_error(&format!("Failed to create shared model: {}", e));
                return Err(e);
            }
        }
    } else {
        log_info(&format!("Shared model already exists at {}", model_path.display()));
    }

    // Get absolute paths
    let model_path_abs = fs::canonicalize(&model_path)?;
    let model_path_str = model_path_abs.to_string_lossy().into_owned();
    
    // Generate settings file
    log_status("Generating settings file...");
    let settings_path = Path::new(PROOF_GEN_DIR).join("settings.json");
    
    let ezkl_bin = which::which("ezkl").map_err(|_| {
        log_error("EZKL command not found in PATH. Make sure EZKL is installed correctly.");
        anyhow::anyhow!("EZKL command not found in PATH. Please install EZKL: https://github.com/zkonduit/ezkl")
    })?;
    
    log_info(&format!("Using EZKL binary at: {}", ezkl_bin.display()));
    
    let output = Command::new(ezkl_bin.clone())
        .arg("gen-settings")
        .arg("-M")
        .arg(&model_path_str)
        .arg("-O")
        .arg(&settings_path)
        .output()
        .context("Failed to execute EZKL gen-settings command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_error(&format!("Failed to generate settings: {}", stderr));
        return Err(anyhow::anyhow!("Failed to generate settings: {}", stderr));
    }
    
    log_success("Settings generated successfully");

    // Create a dummy input.json for calibration
    create_address_input(features, address, PROOF_GEN_DIR)?;

    // Calibrate settings
    log_status("Calibrating settings...");
    
    // First check if input.json exists
    let input_path = format!("{}/input.json", PROOF_GEN_DIR);
    if !Path::new(&input_path).exists() {
        log_error(&format!("Input file not found at: {}", input_path));
        return Err(anyhow::anyhow!("Input file not found at: {}. Make sure address input was created correctly.", input_path));
    }
    
    let output = Command::new(&ezkl_bin)
        .arg("calibrate-settings")
        .arg("-M")
        .arg(&model_path_str)
        .arg("-D")
        .arg(&input_path)
        .arg("-O")
        .arg(&settings_path)
        .output()
        .context("Failed to execute EZKL calibrate-settings command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_error(&format!("Failed to calibrate settings: {}", stderr));
        return Err(anyhow::anyhow!("Failed to calibrate settings: {}", stderr));
    }
    
    // Log the calibration output since it contains useful information
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    
    log_success("Settings calibrated successfully");

    // Download SRS file if needed
    let srs_path = Path::new(PROOF_GEN_DIR).join(SRS_FILE);
    let settings_path = Path::new(PROOF_GEN_DIR).join("settings.json");
    
    // Check if settings.json exists before continuing
    if !settings_path.exists() {
        log_error(&format!("Settings file not found at: {}", settings_path.display()));
        return Err(anyhow::anyhow!("Settings file not found at: {}. Make sure settings generation completed successfully.", settings_path.display()));
    }
    
    if !srs_path.exists() {
        log_status("Downloading SRS file...");
        log_info("This may take a while for large parameters...");
        
        let output = Command::new(&ezkl_bin)
            .arg("get-srs")
            .arg("--settings-path")
            .arg(&settings_path)
            .arg("--srs-path")
            .arg(&srs_path)
            .output()
            .context("Failed to execute EZKL get-srs command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log_error(&format!("Failed to download SRS file: {}", stderr));
            return Err(anyhow::anyhow!("Failed to download SRS file: {}. Check your network connection.", stderr));
        }
        log_success("SRS file downloaded successfully");
    } else {
        log_info(&format!("SRS file already exists at {}", srs_path.display()));
    }

    Ok(())
}

/// Creates address-specific input.json file
pub fn create_address_input(features: &[f32], address: &str, output_dir: &str) -> Result<(), anyhow::Error> {
    log_status(&format!("Creating input for address: {}", address));
    log_info(&format!("Features: {:?}", features));

    // Create output directory if it doesn't exist
    match fs::create_dir_all(output_dir) {
        Ok(_) => log_info(&format!("Directory '{}' is ready", output_dir)),
        Err(e) => {
            log_error(&format!("Failed to create directory '{}': {}", output_dir, e));
            return Err(anyhow::anyhow!("Failed to create directory '{}': {}", output_dir, e));
        }
    };

    // Create the input data in EZKL format with nested arrays
    let ezkl_input = serde_json::json!({
        "input_data": [
            features.to_vec()  // Wrap features in an additional array
        ],
        "input_shapes": [[features.len()]],
        "output_data": [
            [0.0]  // Placeholder output
        ]
    });
    
    // Write the formatted input
    let input_path = Path::new(output_dir).join("input.json");
    fs::write(&input_path, serde_json::to_string_pretty(&ezkl_input)?)
        .context("Failed to write input file")?;

    log_success(&format!("Created input file at: {}", input_path.display()));
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

pub fn create_ezkl_script(script_path: &Path, working_dir: &str, generate_contract: bool) -> Result<(), anyhow::Error> {
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

    // Create a launcher bash script that calls the run_ezkl.sh shell script
    let mut script = if Path::new(SHELL_SCRIPTS[0]).exists() {
        format!(r#"#!/usr/bin/env bash
set -e

# First ensure the shell scripts are executable
chmod +x {} {} {}

# Check if we need to set up common resources
if [ ! -d "shared_circuit" ] || [ ! -f "shared_circuit/model.compiled" ] || [ ! -f "shared_circuit/pk.key" ] || [ ! -f "shared_circuit/vk.key" ]; then
    echo "Setting up common circuit resources..."
    {} "{}" "shared_circuit" "{}"
fi

# Run the individual proof generation
{} {} {}"#,
            SHELL_SCRIPTS[0], SHELL_SCRIPTS[1], SHELL_SCRIPTS[2],
            SHELL_SCRIPTS[1],
            model_path_str,
            srs_path_str,
            SHELL_SCRIPTS[0],
            if generate_contract { "--generate-contract" } else { "" },
            working_dir_str)
    } else {
        return Err(anyhow::anyhow!(
            r#"run_ezkl.sh script not found in the current directory.
Please ensure run_ezkl.sh, run_ezkl_common.sh, and run_ezkl_individual.sh are in the working directory."#
        ));
    };
    
    script.push('\n');

    fs::write(script_path, script).context("Failed to write script file")?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        
        // Set execute permissions on the script and shell scripts
        let scripts_to_chmod = std::iter::once(script_path)
            .chain(SHELL_SCRIPTS.iter().map(Path::new));
        
        for path in scripts_to_chmod {
            if path.exists() {
                let mut perms = fs::metadata(path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(path, perms).context("Failed to set permissions")?;
            }
        }
    }
    
    Ok(())
}

/// Execute the EZKL shell script and process the results
pub fn run_ezkl_process(script_path: &Path) -> Result<(), anyhow::Error> {
    log_status("Processing with EZKL...");
    
    // Execute the script with proper error handling
    let output = Command::new(script_path)
        .output()
        .context("Failed to execute EZKL script")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        log_error("EZKL script execution failed");
        log_error(&format!("stdout: {}", stdout));
        log_error(&format!("stderr: {}", stderr));
        
        return Err(anyhow!("EZKL script failed with status: {}", output.status));
    }
    
    // Print success output
    let stdout = String::from_utf8_lossy(&output.stdout);
    log_success(&format!("EZKL script execution successful:\n{}", stdout));
    Ok(())
}

