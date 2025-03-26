mod proof_registry;
mod script_generator;
mod utils;

use anyhow::{Result, Context};
use std::path::Path;
use std::process::Command;
use std::fs;
use synthetic_data::{
    generate_synthetic_data_with_test_addresses,
    save_data_as_json
};

use crate::proof_registry::create_proof_registry;
use crate::script_generator::{initialize_shared_resources, create_address_input, create_ezkl_script, PROOF_GEN_DIR};
use crate::utils::{address_to_filename, get_features_for_address};

const CONTRACTS_SRC_PATH: &str = "../../contracts/src";
const CONTRACTS_SCRIPT_PATH: &str = "../../contracts/script";

// Test addresses for different tiers
const LOW_TIER_ADDRESS: &str = "0x2222222222222222222222222222222222222222";
const MEDIUM_TIER_ADDRESS: &str = "0x276ef71c8F12508d187E7D8Fcc2FE6A38a5884B1";
const HIGH_TIER_ADDRESS: &str = "0x4444444444444444444444444444444444444444";

fn main() -> Result<()> {
    // Create directories for artifacts
    fs::create_dir_all(PROOF_GEN_DIR)?;
    fs::create_dir_all("script")?;
    fs::create_dir_all("proof_registry")?;

    // Step 1: Generate synthetic data with test addresses
    println!("Generating synthetic data with test addresses...");
    let data = generate_synthetic_data_with_test_addresses(1000)?;
    save_data_as_json(&data, &format!("{}/credit_data.json", PROOF_GEN_DIR))?;

    // Define the addresses to generate proofs for
    let test_addresses = vec![
        LOW_TIER_ADDRESS,
        MEDIUM_TIER_ADDRESS,
        HIGH_TIER_ADDRESS,
    ];

    // Step 2: Generate a shared model once before processing addresses
    println!("Generating shared credit model...");
    let sample_address = test_addresses[0];
    let sample_features = get_features_for_address(&data, sample_address)?;
    initialize_shared_resources(&sample_features, sample_address)?;

    // Step 3: Generate proofs for each test address
    for address in &test_addresses {
        println!("Generating proof for address: {}", address);

        // Create a subdirectory for this address
        let address_dir = format!("{}/{}", PROOF_GEN_DIR, address_to_filename(address));
        fs::create_dir_all(&address_dir)?;

        // Get features for this address
        let address_features = get_features_for_address(&data, address)?;

        // Generate input.json for this address (using the shared model)
        create_address_input(&address_features, address, &address_dir)?;

        // Generate proof with EZKL
        println!("Processing with EZKL...");
        let script_path = Path::new(&address_dir).join("run_ezkl.sh");
        let is_medium_tier = *address == MEDIUM_TIER_ADDRESS;
        create_ezkl_script(&script_path, &address_dir, is_medium_tier)?;

        // Run EZKL script
        let status = Command::new("bash")
            .arg(&script_path)
            .status()
            .context("Failed to execute EZKL script")?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("EZKL script failed with status: {}", status));
        }

        // Create proof registry entry
        println!("Creating proof registry entry...");
        create_proof_registry(address, &address_dir)?;
        println!("Successfully registered proof for address: {}", address);
    }

    // Step 4: Copy artifacts for medium tier address only
    println!("Copying artifacts for Solidity tests...");
    fs::create_dir_all(CONTRACTS_SRC_PATH)?;
    fs::create_dir_all(CONTRACTS_SCRIPT_PATH)?;

    let medium_dir = format!("{}/{}", PROOF_GEN_DIR, address_to_filename(MEDIUM_TIER_ADDRESS));
    fs::copy(
        format!("{}/Halo2Verifier.sol", medium_dir),
        format!("{}/Halo2Verifier.sol", CONTRACTS_SRC_PATH)
    )?;

    fs::copy(
        format!("{}/calldata.json", medium_dir),
        format!("{}/calldata.json", CONTRACTS_SCRIPT_PATH)
    )?;

    println!("Proof generation complete!");
    println!("Generated artifacts:");
    println!(" - Shared credit model in {}/", PROOF_GEN_DIR);
    println!(" - Proofs for each address in {}/address_dir/", PROOF_GEN_DIR);
    println!(" - Medium tier address artifacts copied to contracts repo");

    Ok(())
}