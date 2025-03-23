use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Converts our JSON model to an ONNX format that EZKL can process
#[allow(dead_code)]
pub fn convert_json_to_onnx(
    json_model_path: &Path,
    json_input_path: &Path,
    onnx_model_path: &Path,
    ezkl_input_path: &Path,
) -> Result<()> {
    println!("Converting JSON model to ONNX format...");

    // Create a temporary Python script to do the conversion
    let script_path = json_model_path.parent().unwrap().join("convert_model.py");

    let python_script = r#"
import json
import sys
import torch
import torch.nn as nn
from torch.onnx import export

# Load model from JSON
def load_model_from_json(model_file):
    with open(model_file, 'r') as f:
        model_data = json.load(f)
    return model_data

# Load input data from JSON
def load_input_from_json(input_file):
    with open(input_file, 'r') as f:
        input_data = json.load(f)
    return input_data

# Create a PyTorch model from the JSON coefficients
class LinearModel(nn.Module):
    def __init__(self, coefficients, intercept):
        super(LinearModel, self).__init__()
        self.linear = nn.Linear(len(coefficients), 1)
        
        # Set weights directly from coefficients
        weight = torch.tensor(coefficients).reshape(1, -1)
        self.linear.weight.data = weight
        
        # Set bias from intercept
        self.linear.bias.data = torch.tensor([intercept])
        
        # Add sigmoid to clamp output between 0 and 1
        self.sigmoid = nn.Sigmoid()
    
    def forward(self, x):
        x = self.linear(x)
        x = self.sigmoid(x)
        return x

# Main conversion function
def convert_to_onnx(model_file, input_file, onnx_output, ezkl_input_output):
    # Load the model and input data
    model_data = load_model_from_json(model_file)
    input_data = load_input_from_json(input_file)
    
    coefficients = model_data['coefficients']
    intercept = model_data['intercept']
    
    # Create PyTorch model
    model = LinearModel(coefficients, intercept)
    model.eval()
    
    # Get sample features from input
    sample_features = input_data['sample']['features']
    sample_input = torch.tensor([sample_features], dtype=torch.float32)
    
    # Export to ONNX
    export(
        model, 
        sample_input, 
        onnx_output,
        input_names=['input'], 
        output_names=['output'],
        dynamic_axes={'input': {0: 'batch_size'}, 'output': {0: 'batch_size'}}
    )
    
    # Create EZKL input format
    with torch.no_grad():
        sample_output = model(sample_input)
    
    ezkl_input = {
        "input_shapes": [[len(sample_features)]],
        "input_data": [sample_features],
        "output_data": [sample_output.flatten().tolist()]
    }
    
    with open(ezkl_input_output, 'w') as f:
        json.dump(ezkl_input, f, indent=2)
    
    print(f"Model exported to {onnx_output}")
    print(f"EZKL input saved to {ezkl_input_output}")

if __name__ == "__main__":
    if len(sys.argv) != 5:
        print("Usage: python convert_model.py <model_json> <input_json> <onnx_output> <ezkl_input_output>")
        sys.exit(1)
    
    model_file = sys.argv[1]
    input_file = sys.argv[2]
    onnx_output = sys.argv[3]
    ezkl_input_output = sys.argv[4]
    
    convert_to_onnx(model_file, input_file, onnx_output, ezkl_input_output)
"#;

    fs::write(&script_path, python_script).context("Failed to write Python conversion script")?;

    // Execute the Python script
    println!("Running Python script to convert model...");
    let status = Command::new("python3")
        .arg(&script_path)
        .arg(json_model_path)
        .arg(json_input_path)
        .arg(onnx_model_path)
        .arg(ezkl_input_path)
        .status()
        .context("Failed to execute Python script")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Python script failed with status: {}",
            status
        ));
    }

    // Clean up the temporary script
    let _ = fs::remove_file(script_path);

    Ok(())
}
