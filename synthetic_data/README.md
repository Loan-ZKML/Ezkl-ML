# Synthetic Data Library

This crate provides a library for generating synthetic credit data and training ML models for use in the ZKML credit scoring system.

## Overview

The `synthetic_data` crate is designed to be used as a library, primarily by the `ezkl` crate in this project. It provides functionality for:

1. Generating realistic synthetic credit data
2. Training a simple ML model for credit scoring
3. Exporting the model and sample inputs in a format suitable for ZKML processing

## Synthetic Data Generation

The system generates synthetic credit data that represents on-chain financial activity for Ethereum addresses. The generated data includes the following features:

- **Transaction Count**: Number of transactions (normalized to 0-1 range), representing user activity level
- **Wallet Age**: Age of the wallet in days (normalized to 0-1 range), representing account longevity
- **Average Balance**: Historical average ETH balance (normalized to 0-1 range), representing financial capacity
- **Repayment History**: Binary value (0 or 1) representing previous loan repayment success

Each feature is normalized to the 0-1 range to ensure consistent model training and to work effectively with the ZKML framework. The credit score is computed as a weighted sum of these features, with some added noise for realism:

```
score = 0.3 * tx_count + 0.2 * wallet_age + 0.2 * avg_balance + 0.3 * repayment_history + noise
```

The final score is clamped to the 0-1 range, where 0 represents the lowest credit quality and 1 represents the highest.

### EZKL Scaling for Zero-Knowledge Proofs

For zero-knowledge proof generation, the credit score undergoes two scaling transformations:

1. **Human-readable scaling**: The 0-1 range score is multiplied by 1000 for better readability:
   ```
   scaled_score = score * 1000  // e.g., 0.629 → 629
   ```

2. **EZKL circuit scaling**: The scaled score is multiplied by EZKL's scaling factor (67219) for the ZK circuit:
   ```
   proof_public_input = scaled_score * 67219  // e.g., 629 * 67219 ≈ 42,280,878
   ```

This double scaling allows for both human-readable scores (0-1000) and the integer precision needed for the ZK circuit.

## Model Training

The system trains a simple linear regression model with the following characteristics:

- **Model Type**: Linear regression with sigmoid activation to constrain outputs to the 0-1 range
- **Features**: The four wallet activity metrics described above
- **Output**: A single credit score value between 0 and 1

The model is intentionally simple to facilitate zero-knowledge proof generation, as complex models would result in larger circuits and more computational overhead.

## Collateral Tier System

The credit scores map to collateral tiers as follows:

1. **UNKNOWN Tier** (120% collateral): New user addresses or addresses with no historical data
2. **LOW Tier** (100% collateral): Addresses with scores below 0.4
3. **MEDIUM Tier** (90% collateral): Addresses with scores between 0.4 and 0.7
4. **HIGH Tier** (80% collateral): Addresses with scores above 0.7

## Test Ethereum Addresses

For testing purposes, you should use the following strategies:

### Training Data Addresses

When using addresses that are part of the training data:

- For UNKNOWN tier (120% collateral): Use any new address not in the training set, e.g., `0x1111111111111111111111111111111111111111`
- For LOW tier (100% collateral): Use `0x2222222222222222222222222222222222222222` (score ≈ 0.09 → 90 → 6,049,710)
- For MEDIUM tier (90% collateral): Use `0x276ef71c8F12508d187E7D8Fcc2FE6A38a5884B1` (score ≈ 0.63 → 629 → 42,280,878)
- For HIGH tier (80% collateral): Use `0x4444444444444444444444444444444444444444` (score ≈ 0.86 → 860 → 57,808,340)

### Testing Approach

1. Include these test addresses in the training data with appropriate feature values
2. During proof generation, these addresses should produce proofs that verify their respective tiers
3. For testing new/unknown addresses, use any address not included in the training dataset

It's recommended to maintain these test addresses consistently across development and testing phases to ensure reproducible results.

## Library Usage

This crate is meant to be used as a dependency in other crates:

```rust
// In Cargo.toml
[dependencies]
synthetic_data = { path = "../synthetic_data" }

// In your Rust code
use synthetic_data::{generate_synthetic_data, save_data_as_json};
use synthetic_data::model_trainer::CreditScoreModel;

fn main() -> Result<()> {
    // Generate synthetic data
    let data = generate_synthetic_data(1000)?;
    
    // Train model
    let model = CreditScoreModel::train(&data)?;
    
    // Export model and sample input
    model.export_to_json(Path::new("credit_model.json"))?;
    model.save_sample_input(&data, "input.json")?;
    
    Ok(())
}
```

## Implementation Details

- Uses `ndarray` for matrix operations and model training
- Implements a simple linear regression model with sigmoid activation
- Saves models in JSON format rather than ONNX (conversion happens in the ezkl crate)