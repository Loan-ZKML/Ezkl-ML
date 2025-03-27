use anyhow::{Result, Context};
use std::fs;
use std::path::Path;

/// Creates a proof registry entry for the given address
/// Returns a boolean indicating success
pub fn create_proof_registry(address: &str, proof_dir: &str) -> Result<bool, anyhow::Error> {
    // Read the proof from the proof directory
    let proof_path = format!("{}/proof.json", proof_dir);
    let proof_data = fs::read_to_string(Path::new(&proof_path))
        .context(format!("Failed to read proof data from {}", proof_path))?;
    
    // Store the proof in the registry directory
    let registry_dir = "proof_registry";
    fs::create_dir_all(registry_dir)?;
    
    // Save the proof with the address as the filename
    let registry_path = format!("{}/{}.json", registry_dir, address);
    fs::write(&registry_path, &proof_data)
        .context(format!("Failed to write proof to registry at {}", registry_path))?;
    
    Ok(true)
}
