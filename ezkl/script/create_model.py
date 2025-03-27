import json
import numpy as np
import torch
import torch.nn as nn
from torch.onnx import export
import time
import sys
import os

# Get and validate command line arguments
if len(sys.argv) < 5:
    print("Usage: python3 ./script/create_model.py <output_dir> <address> <features> <generate_model_flag>")
    print("where generate_model_flag is 1 to generate model or 0 to skip model generation")
    sys.exit(1)

output_dir = sys.argv[1]
address = sys.argv[2]
try:
    features = json.loads(sys.argv[3])
    generate_model = sys.argv[4] == "1"
    if not isinstance(features, list) or len(features) != 4:
        raise ValueError("Features must be a list of 4 numbers")
except json.JSONDecodeError:
    print("Error: Features must be a valid JSON array")
    sys.exit(1)
except ValueError as e:
    print(f"Error: {e}")
    sys.exit(1)

os.makedirs(output_dir, exist_ok=True)

# Define model that matches Rust implementation
class CreditScoreModel(nn.Module):
    def __init__(self):
        super(CreditScoreModel, self).__init__()
        # Match Rust weights [0.3, 0.2, 0.2, 0.3]
        self.weights = nn.Parameter(torch.tensor([[0.3, 0.2, 0.2, 0.3]]).float())
        
    def forward(self, x):
        # Linear combination of features
        raw_score = torch.matmul(x, self.weights.t())
        # Apply the same transformation as Rust: sigmoid(10.0 * x - 5.0)
        # Fix the order of operations to match Rust
        scaled_input = 10.0 * raw_score - 5.0
        scaled_score = 1.0 / (1.0 + torch.exp(-scaled_input))
        return scaled_score

# Create and evaluate model
model = CreditScoreModel()
model.eval()

# Calculate score
input_tensor = torch.tensor([features], dtype=torch.float32)
with torch.no_grad():
    score = model(input_tensor).item()

print(f"Address: {address}")
print(f"Features: {features}")
print(f"Calculated score: {score:.4f}")

# Calculate tier based on score
if score < 0.4:
    tier = "LOW"
elif score < 0.7:
    tier = "MEDIUM"
else:
    tier = "HIGH"

print(f"Credit tier: {tier}")
print(f"Threshold for favorable rate: 0.5")
print(f"Qualifies for favorable rate (100% collateral): {score > 0.5}")

# Export to ONNX if requested
model_path = os.path.join(output_dir, "credit_model.onnx")
if generate_model:
    print(f"Generating model file: {model_path}")
    export(
        model,
        input_tensor,
        model_path,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={"input": {0: "batch_size"}, "output": {0: "batch_size"}}
    )
else:
    print("Skipping model generation as per flag")

# Scale score for EZKL (0-1000 range)
scaled_score = int(score * 1000)
print(f"Scaled score (0-1000): {scaled_score}")

# Prepare EZKL input
ezkl_input = {
    "input_shapes": [[4]],
    "input_data": [features],
    "output_data": [[score]],
    "public_output_idxs": [[0, 0]]
}

# Save EZKL input
input_path = os.path.join(output_dir, "input.json")
with open(input_path, "w") as f:
    json.dump(ezkl_input, f, indent=2)

# Save debug information
debug_info = {
    "address": address,
    "features": features,
    "original_score": score,
    "scaled_score": scaled_score,
    "credit_tier": tier,
    "favorable_rate_eligible": score > 0.5,
    "model_weights": model.weights.tolist()[0],
    "timestamp": int(time.time())
}

debug_path = os.path.join(output_dir, "scaling_debug.json")
with open(debug_path, "w") as f:
    json.dump(debug_info, f, indent=2)

# Save metadata
metadata = {
    "address": address,
    "features": features,
    "score": score,
    "scaled_score": scaled_score,
    "timestamp": int(time.time()),
    "model_version": "1.0.0"
}

metadata_path = os.path.join(output_dir, "metadata.json")
with open(metadata_path, "w") as f:
    json.dump(metadata, f, indent=2)

if generate_model:
    print(f"Model converted to ONNX and input prepared for EZKL in {output_dir}")
else:
    print(f"Input prepared for EZKL in {output_dir}")
