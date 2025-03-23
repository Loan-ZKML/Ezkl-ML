use anyhow::Result;
use synthetic_data::CreditData;

pub fn address_to_filename(address: &str) -> String {
    // Remove '0x' prefix if present and return the address
    address.trim_start_matches("0x").to_string()
}

pub fn get_features_for_address(data: &CreditData, address: &str) -> Result<Vec<f32>> {
    if let Some(ref address_mapping) = data.address_mapping {
        if let Some(&index) = address_mapping.get(address) {
            return Ok(data.features[index].clone());
        }
    }

    // If not found, use default features (this shouldn't happen with our test addresses)
    Err(anyhow::anyhow!("Address not found in synthetic data: {}", address))
}
