fn create_proof_registry(address: &str, dir_path: &str) -> Result<String> {
    // Calculate proof hash
    let proof_data = fs::read(format!("{}/proof.json", dir_path))?;
    let mut hasher = Sha256::new();
    hasher.update(&proof_data);
    let result = hasher.finalize();
    let proof_hash = hex::encode(result);

    // Load the proof for inspection
    let proof_json = fs::read_to_string(format!("{}/proof.json", dir_path))?;
    let proof: serde_json::Value = serde_json::from_str(&proof_json)?;

    // Extract public input from the proof
    let public_input = if let Some(instances) = proof.get("instances") {
        if let Some(instance_array) = instances.as_array() {
            if !instance_array.is_empty() {
                if let Some(value_str) = instance_array[0][0].as_str() {
                    // Remove "0x" prefix if present
                    let hex_str = value_str.trim_start_matches("0x");

                    // Group into pairs of characters (bytes)
                    let byte_pairs: Vec<&str> = hex_str
                        .as_bytes()
                        .chunks(2)
                        .map(std::str::from_utf8)
                        .collect::<Result<Vec<&str>, _>>()
                        .unwrap_or_default();

                    // Reverse the bytes (because it's little-endian)
                    let reversed_hex = byte_pairs.into_iter().rev().collect::<String>();

                    // Convert from hex to decimal
                    u64::from_str_radix(&reversed_hex, 16).unwrap_or(0) as u32
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };
    
    println!("Extracted public input from proof: {}", public_input);

    // For debugging, also extract the value from witness.json
    let witness_data = fs::read_to_string(format!("{}/witness.json", dir_path))?;
    let _witness: serde_json::Value = serde_json::from_str(&witness_data)?;

    // Log the original and scaled values found in metadata
    let metadata_data = fs::read_to_string(format!("{}/metadata.json", dir_path))?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata_data)?;

    let original_score = metadata["score"].as_f64().unwrap_or(0.0);
    let scaled_score = metadata["scaled_score"].as_u64().unwrap_or(0) as u32;

    println!("Original score from metadata: {}", original_score);
    println!("Scaled score from metadata: {}", scaled_score);
    println!("Public input value in proof: {}", public_input);

    // Create a debug file to track the scaling issue
    let scaling_debug = serde_json::json!({
        "address": address,
        "original_score": original_score,
        "scaled_score_1000": scaled_score,
        "proof_public_input": public_input,
        "ezkl_scaling_factor": if scaled_score > 0 { public_input as f64 / scaled_score as f64 } else { 0.0 },
    });

    fs::write(format!("{}/scaling_analysis.json", dir_path),
            serde_json::to_string_pretty(&scaling_debug)?)?;

    // Create registry entry using the public input value
    let registry_entry = ProofMetadata {
        proof_hash: proof_hash.clone(),
        credit_score: public_input,  // Use the actual public input from the proof
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        model_version: "1.0.0".to_string(),
        address: address.to_string(),
    };

    // Save registry entry
    let registry_path = format!("proof_registry/{}.json", address_to_filename(address));
    fs::write(&registry_path, serde_json::to_string_pretty(&registry_entry)?)?;
    
    // Create lookup file for testing
    let lookup = serde_json::json!({
        "address": address,
        "proof_hash": registry_entry.proof_hash,
        "credit_score": registry_entry.credit_score,
        "public_input": format!("0x{:x}", registry_entry.credit_score),
        "original_score": original_score,
        "ezkl_scaling_debug": {
            "proof_public_input": public_input,
            "metadata_scaled_score": scaled_score,
            "scaling_factor": if scaled_score > 0 { public_input as f64 / scaled_score as f64 } else { 0.0 }
        }
    });

    // Save lookup file
    fs::write(format!("{}/lookup.json", dir_path),
            serde_json::to_string_pretty(&lookup)?)?;

    Ok(proof_hash)
}
