//! This is an example showing how to use the synthetic_data library.
//! For actual usage, see the ezkl crate which uses this library
//! as part of its workflow.

use anyhow::Result;
use std::path::Path;

use synthetic_data::model_trainer::CreditScoreModel;
use synthetic_data::{generate_synthetic_data, save_data_as_json};

fn main() -> Result<()> {
    // main provided for testing purposes.
    // In the actual project, this functionality is used by the ezkl crate.
    println!("EXAMPLE: Using synthetic_data library");
    println!("Note: This main() tests this library. In practice, use the ezkl crate.");
    println!("------------------------------------------------------------");

    // Generate synthetic data
    println!("1. Generating synthetic credit data...");
    let data = generate_synthetic_data(1000)?;
    save_data_as_json(&data, "example_credit_data.json")?;

    // Train model
    println!("2. Training credit score model...");
    let model = CreditScoreModel::train(&data)?;

    // Export model to JSON
    println!("3. Exporting model to JSON format...");
    model.export_to_json(Path::new("example_credit_model.json"))?;

    // Save sample input for EZKL
    println!("4. Saving sample input for EZKL...");
    model.save_sample_input(&data, "example_input.json")?;

    println!("------------------------------------------------------------");
    println!("Example completed. Generated files with 'example_' prefix.");
    println!("For the actual workflow, please run the ezkl crate:");
    println!("cd ../ezkl && cargo run");

    Ok(())
}
