# EZKL Integration for Credit Scoring

This crate serves as the main entry point for the ZKML credit scoring system. It:

1. Uses the `synthetic_data` library to generate data and train ML models
2. Converts the JSON model to ONNX format
3. Integrates with the EZKL CLI tool to create zero-knowledge proofs
4. Generates Solidity contracts for on-chain verification

This crate handles the complete pipeline from data generation to proof creation in one step.

## How It Works

1. The system generates a linear regression model with synthetic credit data
2. The model is saved in JSON format with coefficients and intercept
3. A Python converter transforms this JSON model into ONNX format for EZKL
4. The EZKL tool processes the ONNX model to create ZK circuits and proofs
5. A verifier contract is generated for on-chain verification

## Credit Scoring Model for DeFi Loans

The EZKL integration uses a model trained on synthetic data to predict credit scores for Ethereum addresses. This allows DeFi protocols to implement collateral tiers while maintaining user privacy.

### EZKL Scaling Factor

The system uses EZKL's scaling factor of 67219 to convert floating-point scores to integers for the zero-knowledge circuit:
- Original ML model output: 0.0-1.0 range (e.g., 0.629)
- Human-readable scale: 0-1000 range (e.g., 629)
- EZKL proof public input: score * 67219 (e.g., 42,280,878)

This scaling is required because ZK circuits work with integers, not floating-point numbers.

### Model Details

The credit scoring model has the following characteristics:

- **Model Architecture**: Linear regression with sigmoid activation
- **Input Features**:
  - Transaction count (normalized)
  - Wallet age (normalized)
  - Average balance (normalized)
  - Repayment history (binary)
- **Output**: A credit score between 0-1

### Collateral Tier System

The system implements a tiered collateral structure based on credit scores:

1. **UNKNOWN Tier** (120% collateral): For addresses with no verifiable history or proof
2. **LOW Tier** (100% collateral): For addresses with scores below 0.4
3. **MEDIUM Tier** (90% collateral): For addresses with scores between 0.4 and 0.7
4. **HIGH Tier** (80% collateral): For addresses with scores above 0.7

### Ethereum Address Test Strategy

For testing the system, using specific Ethereum addresses consistently:

- **UNKNOWN Tier**: Any address not in the training data, e.g., `0x1111111111111111111111111111111111111111`
- **LOW Tier**: `0x2222222222222222222222222222222222222222`
- **MEDIUM Tier**: `0x276ef71c8F12508d187E7D8Fcc2FE6A38a5884B1` (prv-key: `0x08c216a5cbe31fd3c8095aae062a101c47c0f6110d738b97c5b1572993a2e665`)
- **HIGH Tier**: `0x4444444444444444444444444444444444444444`

When training the model and generating proofs:
1. Include these test addresses in the training data with appropriate feature values
2. During proof generation, use these addresses to ensure they produce proofs that verify their respective tiers
3. For testing new/unknown users, use any random address not included in the training data

## EZKL Command Reference (for EZKL version 20.2.0)

```bash
# Convert JSON model to ONNX format (using PyTorch)
python convert_model.py credit_model.json input.json credit_model.onnx ezkl_input.json

# Generate settings
ezkl gen-settings -M credit_model.onnx -O settings.json

# Calibrate settings
ezkl calibrate-settings -M credit_model.onnx -D ezkl_input.json -O settings.json

# Compile model to circuit
ezkl compile-circuit -M credit_model.onnx --compiled-circuit model.compiled -S settings.json

# Generate keys
ezkl setup -M model.compiled --pk-path pk.key --vk-path vk.key --srs-path kzg.srs

# Generate witness
ezkl gen-witness -D ezkl_input.json -M model.compiled -O witness.json

# Generate proof
ezkl prove --witness witness.json --proof-path proof.json --pk-path pk.key --compiled-circuit model.compiled --srs-path kzg.srs

# Verify proof
ezkl verify --proof-path proof.json --vk-path vk.key --srs-path kzg.srs

# Create verifier contract
ezkl create-evm-verifier --vk-path vk.key --sol-code-path Halo2Verifier.sol --srs-path kzg.srs

# Generate calldata
ezkl encode-evm-calldata --proof-path proof.json --calldata-path calldata.json
```

## Prerequisites

- Install the EZKL CLI tool: https://github.com/zkonduit/ezkl
- Ensure `ezkl` is in your system PATH
- Python dependencies:
  ```bash
  pip install torch numpy onnx
  ```

  Note: PyTorch (torch) is required for creating the model and exporting to ONNX format. The ONNX package is needed for the conversion process.

## Usage

### Option 1: Run the Rust binary

First, make sure you're in the ezkl directory:

```bash
cd /path/to/loan-zkml/demo/ezkl
```

Then run the binary:

```bash
cargo run
```

This implementation has two methods of executing EZKL:

1. Using the shell script `run_ezkl.sh` (preferred method)
2. Using a direct Rust implementation as fallback (if the script execution fails)

Both methods achieve the same result - processing the model with EZKL to generate ZK proofs.

This will:
1. Generate synthetic credit data
2. Train a credit scoring model
3. Export the model to JSON format
4. Save sample input for EZKL
5. Execute the `run_ezkl.sh` script which uses the real EZKL binary to:
   - Generate circuit settings
   - Compile the model into a circuit
   - Set up proving and verification keys
   - Generate and verify proofs
   - Create a Solidity verifier contract

### Option 2: Run the shell script directly

If you've already generated the synthetic data and model, you can run the script directly:

```bash
./run_ezkl.sh
```

## Generated Artifacts

The process generates the following files, organized by Ethereum address:

```
proof_generation/
├── <ethereum_address>/              # Address-specific directories
│   ├── credit_model.onnx            # ONNX model
│   ├── input.json                   # Model input
│   ├── metadata.json                # Original and scaled scores
│   ├── lookup.json                  # Original, scaled, and proof scores
│   ├── proof.json                   # ZK proof
│   ├── scaling_analysis.json        # EZKL scaling details
│   ├── settings.json                # EZKL settings
│   ├── witness.json                 # ZK witness
│   ├── model.compiled               # Compiled circuit
│   ├── pk.key                       # Proving key
│   ├── vk.key                       # Verification key
│   ├── kzg.srs                      # Structured Reference String
│   └── calldata.json                # EVM calldata (medium tier only)
├── credit_data.json                 # Generated synthetic data

proof_registry/
└── <ethereum_address>.json          # Proof registry entries with:
                                     # - proof_hash
                                     # - credit_score (EZKL scaled)
                                     # - timestamp
                                     # - model_version
                                     # - address
```

The directory structure separates proofs by Ethereum address for better organization.

## Integration with Smart Contracts

The generated Solidity verifier contract (`Halo2Verifier.sol`) can be deployed to Ethereum or other EVM-compatible blockchains. This contract verifies zero-knowledge proofs of credit scores without revealing the underlying data.

For loan contract integration, see the sample interfaces in `docs/overview.md`.
