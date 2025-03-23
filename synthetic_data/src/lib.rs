//! Synthetic data generation and model training for credit score prediction.
//!
//! This library provides functions for generating synthetic credit data and
//! training machine learning models for credit scoring. It's primarily used
//! by the ezkl crate in this project to create data for ZKML demonstrations.
//!
//! # Basic Example
//!
//! ```no_run
//! use synthetic_data::{generate_synthetic_data, save_data_as_json};
//! use synthetic_data::model_trainer::CreditScoreModel;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate synthetic data
//! let data = generate_synthetic_data(1000)?;
//!
//! // Train model
//! let model = CreditScoreModel::train(&data)?;
//!
//! // Export model and sample input
//! model.export_to_json(Path::new("credit_model.json"))?;
//! model.save_sample_input(&data, "input.json")?;
//! # Ok(())
//! # }
//! ```
//!
//! # Example with Ethereum Address Tiers
//!
//! ```no_run
//! use synthetic_data::{generate_synthetic_data_with_test_addresses, add_address_to_data, save_data_as_json};
//! use synthetic_data::model_trainer::CreditScoreModel;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate synthetic data with test addresses for collateral tiers
//! let mut data = generate_synthetic_data_with_test_addresses(1000)?;
//!
//! // Add more addresses with specific features
//! // Format: [tx_count, wallet_age, avg_balance, repayment_history]
//! // This will result in another address qualified for HIGH tier (>0.7 score)
//! add_address_to_data(&mut data, "0x6666666666666666666666666666666666666666", vec![0.95, 0.9, 0.8, 1.0])?;
//!
//! // Add a custom address with specific credit score (around 0.5, MEDIUM tier)
//! add_address_to_data(&mut data, "0x7777777777777777777777777777777777777777", vec![0.4, 0.5, 0.5, 0.5])?;
//!
//! // Train model
//! let model = CreditScoreModel::train(&data)?;
//!
//! // Export model and sample input
//! model.export_to_json(Path::new("credit_model.json"))?;
//! model.save_sample_input(&data, "input.json")?;
//! # Ok(())
//! # }
//! ```

pub mod model_trainer;
pub mod synthetic_data;

// Re-export commonly used items for convenience
pub use model_trainer::CreditScoreModel;
pub use synthetic_data::add_address_to_data;
pub use synthetic_data::generate_synthetic_data;
pub use synthetic_data::generate_synthetic_data_with_test_addresses;
pub use synthetic_data::save_data_as_json;
pub use synthetic_data::CreditData;
