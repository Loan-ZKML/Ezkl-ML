use anyhow::Result;
use rand::thread_rng;
use rand_distr::{Distribution, Normal, Uniform};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
/// Sigmoid activation function that maps any real number to a value between 0 and 1
///
/// Transforms inputs using the formula: f(x) = 1 / (1 + e^(-x))
/// The function has a natural S-curve shape and smoothly maps all real inputs to [0, 1]
/// Handles extreme values to prevent floating-point issues.
fn sigmoid(x: f32) -> Option<f32> {
    // Handle extreme values to avoid floating-point issues
    // Using a smaller threshold (10.0) to ensure more precise bounds for extreme values
    const EPSILON: f32 = 1e-6f32;

    if !x.is_finite() {
        // Handle NaN and infinity
        None
    } else {
        let result = if x > 10.0f32 {
            // For large positive inputs, return a value very close to 1.0 but not exactly 1.0
            1.0f32 - EPSILON
        } else if x < -10.0f32 {
            // For large negative inputs, return a value very close to 0.0 but not exactly 0.0
            EPSILON
        } else {
            // Normal calculation for reasonable range inputs
            1.0f32 / (1.0f32 + (-x).exp())
        };
        Some(result)
    }
}

/// Validates a single feature value, ensuring it's finite and within the [0,1] range
fn validate_feature(x: f32) -> Option<f32> {
    if x.is_finite() {
        Some(x.clamp(0.0f32, 1.0f32))
    } else {
        None
    }
}

/// Validates all features in a slice, ensuring each is valid
/// Returns None if any feature is invalid
fn validate_features(features: &[f32]) -> Option<Vec<f32>> {
    features.iter().map(|&x| validate_feature(x)).collect()
}

/// Calculates a weighted score from validated features
/// Returns None if there aren't exactly 4 features
fn calculate_weighted_score(features: &[f32]) -> Option<f32> {
    if features.len() != 4 {
        return None;
    }

    let weights = [0.3f32, 0.2f32, 0.2f32, 0.3f32];
    Some(
        features
            .iter()
            .zip(weights.iter())
            .map(|(&f, &w)| f * w)
            .sum(),
    )
}

/// Processes noise to ensure it's within reasonable bounds
fn process_noise(noise: f32) -> Option<f32> {
    if noise.is_finite() {
        Some(noise.clamp(-5.0f32, 5.0f32))
    } else {
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditData {
    pub features: Vec<Vec<f32>>,
    pub scores: Vec<f32>,
    pub feature_names: Vec<String>,
    // Optional mapping from Ethereum addresses to indices in the features/scores arrays
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_mapping: Option<HashMap<String, usize>>,
}

pub fn generate_synthetic_data(num_samples: usize) -> Result<CreditData> {
    let mut rng = thread_rng();

    // Define distributions with their normalizers in a single array for better organization
    let distributions = [
        (Uniform::new(0.0f32, 5000.0f32), 5000.0f32),
        (Uniform::new(0.0f32, 365.0f32 * 5.0f32), 365.0f32 * 5.0f32),
        (Uniform::new(0.0f32, 10000.0f32), 10000.0f32),
        (Uniform::new(0.0f32, 1.0f32), 1.0f32),
    ];

    // Generate features using iterators in a more compact way
    let features: Vec<Vec<f32>> = (0..num_samples)
        .map(|_| {
            distributions
                .iter()
                .map(|(dist, normalizer)| {
                    let raw_value = dist.sample(&mut rng);
                    // For the last feature (repayment history), round to 0 or 1
                    if normalizer == &1.0f32 {
                        // Just check the normalizer for repayment history
                        f32::round(raw_value)
                    } else {
                        raw_value / normalizer
                    }
                })
                .collect()
        })
        .collect();

    // Generate synthetic credit scores as a function of features with some noise
    let noise_dist = Normal::new(0.0f32, 0.05f32)?;

    // Calculate scores using the new pure functions
    let scores: Vec<f32> = features
        .iter()
        .map(|feature| {
            let noise = noise_dist.sample(&mut rng);
            calculate_score(feature, noise)
        })
        .collect();

    let feature_names = vec![
        "tx_count".to_string(),
        "wallet_age".to_string(),
        "avg_balance".to_string(),
        "repayment_history".to_string(),
    ];

    Ok(CreditData {
        features,
        scores,
        feature_names,
        address_mapping: None,
    })
}

/// Calculate a credit score based on feature values using a composition of pure functions
fn calculate_score(features: &[f32], noise: f32) -> f32 {
    // Use a pipeline of pure functions with Option type for proper error handling
    let result = validate_features(features)
        .and_then(|valid_features| calculate_weighted_score(&valid_features))
        .and_then(|score| process_noise(noise).map(|n| score + n))
        .and_then(|combined| sigmoid(10.0f32 * combined - 5.0f32));

    // Default score for invalid inputs
    result.unwrap_or(0.5f32)
}

/// Generates synthetic data with specific test addresses included
///
/// This function is similar to `generate_synthetic_data` but adds specific test addresses
/// with predetermined features to ensure they map to specific credit score tiers.
/// This is useful for testing the collateral tier system.
pub fn generate_synthetic_data_with_test_addresses(num_samples: usize) -> Result<CreditData> {
    let mut data = generate_synthetic_data(num_samples)?;

    let low_tier_address = "0x2222222222222222222222222222222222222222";
    let medium_tier_address = "0x276ef71c8F12508d187E7D8Fcc2FE6A38a5884B1";
    let high_tier_address = "0x4444444444444444444444444444444444444444";
    let another_high_tier_address = "0x5555555555555555555555555555555555555555";

    // LOW tier address (score < 0.4)
    let low_tier_features = vec![0.1f32, 0.2f32, 0.1f32, 0.0f32];

    // MEDIUM tier address (score between 0.4 and 0.7)
    let medium_tier_features = vec![0.5f32, 0.4f32, 0.5f32, 1.0f32];

    // HIGH tier address (score > 0.7)
    let high_tier_features = vec![0.9f32, 0.8f32, 0.7f32, 1.0f32];

    // Another HIGH tier address
    let another_high_tier_features = vec![0.85f32, 0.9f32, 0.8f32, 1.0f32];

    add_address_to_data(&mut data, low_tier_address, low_tier_features)?;
    add_address_to_data(&mut data, medium_tier_address, medium_tier_features)?;
    add_address_to_data(&mut data, high_tier_address, high_tier_features)?;
    add_address_to_data(
        &mut data,
        another_high_tier_address,
        another_high_tier_features,
    )?;

    Ok(data)
}

/// Add a specific Ethereum address to the dataset with custom features
///
/// This allows adding any number of addresses to the dataset with specific
/// features that will result in desired credit scores.
pub fn add_address_to_data(data: &mut CreditData, address: &str, features: Vec<f32>) -> Result<()> {
    // Ensure we have an address mapping
    if data.address_mapping.is_none() {
        data.address_mapping = Some(HashMap::new());
    }

    // Calculate score for the features
    let score = calculate_score(&features, 0.0);

    // Add to the dataset
    let index = data.features.len();
    data.features.push(features);
    data.scores.push(score);

    // Update address mapping
    if let Some(ref mut mapping) = data.address_mapping {
        mapping.insert(address.to_string(), index);
    }

    Ok(())
}

pub fn save_data_as_json(data: &CreditData, path: &str) -> Result<()> {
    let json_data = serde_json::to_string_pretty(data)?;
    let mut file = File::create(path)?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test module for verifying the functionality of the synthetic data generation module.
    ///
    /// These tests ensure:
    /// 1. The sigmoid function behaves as expected with proper output range,
    ///    midpoint at 0, and symmetry properties.
    /// 2. Credit score calculations work correctly for normal and edge cases.
    /// 3. The score remains within the expected [0, 1] range regardless of inputs.

    /// Tests basic properties of the sigmoid function
    #[test]
    fn test_sigmoid_properties() {
        // Test output range (should always be between 0 and 1)
        assert!(sigmoid(-100.0f32).unwrap() > 0.0f32 && sigmoid(-100.0f32).unwrap() < 1.0f32);
        assert!(sigmoid(100.0f32).unwrap() > 0.0f32 && sigmoid(100.0f32).unwrap() < 1.0f32);

        // Test extremes approach limits but never reach them
        let epsilon = 1e-6f32;
        assert!(sigmoid(-100.0f32).unwrap() >= epsilon); // Very close to 0 but not exactly 0
        assert!(sigmoid(100.0f32).unwrap() <= 1.0f32 - epsilon); // Very close to 1 but not exactly 1

        // Test midpoint property (f(0) = 0.5)
        assert!((sigmoid(0.0f32).unwrap() - 0.5f32).abs() < 1e-6f32);

        // Test symmetry property (f(-x) = 1 - f(x))
        for x in [-5.0f32, -1.0f32, 0.0f32, 1.0f32, 5.0f32].iter() {
            assert!((sigmoid(-*x).unwrap() - (1.0f32 - sigmoid(*x).unwrap())).abs() < 1e-6f32);
        }

        // Test None case
        assert!(sigmoid(f32::NAN).is_none());
        assert!(sigmoid(f32::INFINITY).is_none());
        assert!(sigmoid(f32::NEG_INFINITY).is_none());
    }

    /// Tests credit score calculation with normal input values
    #[test]
    fn test_credit_score_normal_cases() {
        // Low score features
        let low_features = [0.1f32, 0.2f32, 0.1f32, 0.0f32];
        let low_score = calculate_score(&low_features, 0.0f32);
        assert!(low_score < 0.4f32);

        // Medium score features
        let medium_features = [0.5f32, 0.4f32, 0.5f32, 0.5f32];
        let medium_score = calculate_score(&medium_features, 0.0f32);
        assert!(medium_score >= 0.4f32 && medium_score <= 0.7f32);

        // High score features
        let high_features = [0.9f32, 0.8f32, 0.7f32, 1.0f32];
        let high_score = calculate_score(&high_features, 0.0);
        assert!(high_score > 0.7);

        // Verify that noise affects the score
        let score_with_noise = calculate_score(&medium_features, 0.2);
        let score_without_noise = calculate_score(&medium_features, 0.0);
        assert_ne!(score_with_noise, score_without_noise);
    }

    /// Tests credit score calculation with edge cases
    #[test]
    fn test_credit_score_edge_cases() {
        // All zeros
        let zero_features = [0.0, 0.0, 0.0, 0.0];
        let zero_score = calculate_score(&zero_features, 0.0);
        assert!(zero_score > 0.0);

        // All ones (maximum possible features)
        let max_features = [1.0, 1.0, 1.0, 1.0];
        let max_score = calculate_score(&max_features, 0.0);
        assert!(max_score < 1.0);
        // Very large positive noise shouldn't push score above 1.0
        let score_large_noise = calculate_score(&zero_features, 100.0);
        assert!(score_large_noise < 1.0);
        assert!(score_large_noise > 0.8); // Should be reasonably close to 1.0 but not exactly 1.0

        // Very large negative noise shouldn't push score below 0.0
        let score_negative_noise = calculate_score(&max_features, -100.0);
        assert!(score_negative_noise > 0.0);
        assert!(score_negative_noise < 0.2); // Should be reasonably close to 0.0 but not exactly 0.0

        // Test with non-finite noise
        let nan_score = calculate_score(&max_features, f32::NAN);
        assert!(nan_score.is_finite()); // Result should be finite even with NaN input

        let inf_score = calculate_score(&max_features, f32::INFINITY);
        assert!(inf_score.is_finite()); // Result should be finite even with infinity input
        assert!(score_negative_noise > 0.0);
    }

    /// Tests the generate_synthetic_data function
    #[test]
    fn test_generate_synthetic_data() {
        let num_samples = 10;
        let result = generate_synthetic_data(num_samples);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.features.len(), num_samples);
        assert_eq!(data.scores.len(), num_samples);

        // All scores should be in range [0, 1]
        for score in data.scores.iter() {
            assert!(*score >= 0.0 && *score <= 1.0);
        }

        // Check dimensions of features
        for feature_vec in data.features.iter() {
            assert_eq!(feature_vec.len(), 4);
        }
    }
}
