import json
import numpy as np
import torch
import torch.nn as nn
from torch.onnx import export
import time
import sys
import os

# Get and validate command line arguments
if len(sys.argv) < 4:
    print("Usage: python3 create_model.py <output_dir> <address> <features>")
    sys.exit(1)

output_dir = sys.argv[1]
address = sys.argv[2]
try:
    features = json.loads(sys.argv[3])
    if not isinstance(features, list) or len(features) != 4:
        raise ValueError("Features must be a list of 4 numbers")
except json.JSONDecodeError:
    print("Error: Features must be a valid JSON array")
    sys.exit(1)
except ValueError as e:
    print(f"Error: {e}")
    sys.exit(1)

# Use output directory directly without creating an additional subdirectory
os.makedirs(output_dir, exist_ok=True)

# Define a model that uses linear scaling instead of sigmoid
class LinearCreditScoreModel(nn.Module):
    def __init__(self):
        super(LinearCreditScoreModel, self).__init__()
        # Define weights for credit scoring features
        self.weights = nn.Parameter(torch.tensor([[0.25, 0.20, 0.25, 0.30]]).float())
        self.bias = nn.Parameter(torch.tensor([0.0]))
        
    def forward(self, x):
        # Linear combination of features
        raw_score = torch.matmul(x, self.weights.t()) + self.bias
        scaled_score = torch.clamp(raw_score, 0.0, 1.0)
        return scaled_score

# Create the model
model = LinearCreditScoreModel()
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

# Export to ONNX
model_path = os.path.join(output_dir, "credit_model.onnx")
export(
    model,
    input_tensor,
    model_path,
    input_names=["input"],
    output_names=["output"],
    dynamic_axes={"input": {0: "batch_size"}, "output": {0: "batch_size"}}
)

# Use direct scaling to avoid potential EZKL quirks
scaled_score = int(score * 1000)
print(f"Scaled score (0-1000): {scaled_score}")

# For EZKL input
ezkl_input = {
    "input_shapes": [[4]],
    "input_data": [features],
    "output_data": [[score]],
    "public_output_idxs": [[0, 0]]
}

input_path = os.path.join(output_dir, "input.json")
with open(input_path, "w") as f:
    json.dump(ezkl_input, f, indent=2)

# Save debug file
debug_info = {
    "address": address,
    "features": features,
    "original_score": score,
    "scaled_score": scaled_score,
    "credit_tier": tier,
    "favorable_rate_eligible": score > 0.5,
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

print(f"Model converted to ONNX and input prepared for EZKL in {output_dir}")
