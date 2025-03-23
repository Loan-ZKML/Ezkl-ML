
import os
import json
import numpy as np
import torch
import torch.nn as nn
from torch.onnx import export
import time

# Define a simple model - linear with sigmoid
class SimpleModel(nn.Module):
    def __init__(self):
        super(SimpleModel, self).__init__()
        # Simple linear model with sigmoid activation
        self.linear = nn.Linear(4, 1)

        # Set weights directly - this is a simplified version of the model
        # We use weights that match a simple credit scoring formula
        weight = torch.tensor([[0.3, 0.2, 0.2, 0.3]]).float()
        self.linear.weight.data = weight
        self.linear.bias.data = torch.tensor([0.0])

        # Sigmoid to normalize output between 0 and 1
        self.sigmoid = nn.Sigmoid()

    def forward(self, x):
        x = self.linear(x)
        x = self.sigmoid(x)
        return x

# Create the model
model = SimpleModel()
model.eval()

# Define features for credit score calculation
favorable_features = [0.8, 0.7, 0.6, 1.0]  # [tx_count, wallet_age, avg_balance, repayment_history]

# Calculate score
input_tensor = torch.tensor([favorable_features], dtype=torch.float32)
with torch.no_grad():
    score = model(input_tensor).item()

print(f"Features: {favorable_features}")
print(f"Calculated score: {score:.4f}")
print(f"Threshold for favorable rate: 0.5")
print(f"Qualifies for favorable rate (100% collateral): {score > 0.5}")

os.makedirs("proof_generation", exist_ok=True)

# Export to ONNX
export(
    model,
    input_tensor,
    "proof_generation/credit_model.onnx",
    input_names=["input"],
    output_names=["output"],
    dynamic_axes={"input": {0: "batch_size"}, "output": {0: "batch_size"}}
)

# Create EZKL input
# Scale the score to be between 0-1000 for easier integer comparison in smart contracts
scaled_score = int(score * 1000)
print(f"Scaled score (0-1000): {scaled_score}")

ezkl_input = {
  "input_shapes": [[4]],
  "input_data": [favorable_features],
  # This is important - we include the scaled score as a public output
  # This will become a public input to the verification system
  "output_data": [[scaled_score / 1000.0]]
}

with open("proof_generation/input.json", "w") as f:
    json.dump(ezkl_input, f, indent=2)

# Also save metadata for later reference
metadata = {
    "features": favorable_features,
    "score": score,
    "scaled_score": scaled_score,
    "timestamp": int(time.time()),
    "model_version": "1.0.0"
}

with open("proof_generation/metadata.json", "w") as f:
    json.dump(metadata, f, indent=2)

print("Model converted to ONNX and input prepared for EZKL")
