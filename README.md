[![Rust CI](https://github.com/loan-zkml/demo/actions/workflows/ci.yml/badge.svg)](https://github.com/loan-zkml/demo/actions/workflows/ci.yml)

# ZKML Credit Scoring System for DeFi Loans

This project implements a privacy-preserving credit scoring system for DeFi loans using zero-knowledge (Halo2) machine learning (ZKML).
It combines the ML creation and synthetic historical loan transaction history training with Ethereum [Loan-ZKML/contracts](https://github.com/Loan-ZKML/contracts) Loan smart contracts to verify the ML computations that result in discounted loan collateral requirements.

## Note on 3rd-Party Development Tools

Look at the file [./.tool-versions](./.tool-versions) to find out which 3rd-party development tools we are using.
Make sure that you have the corresponding tools and versions installed and used by your working shell.

You may want to use [asdf](https://asdf-vm.com/) to manage different versions of these tools.

## Rust Development Setup

You should install the version of Rust specified inside the [./.tool-versions](./.tool-versions) file.

In order to make sure that Rust source code files are properly and uniformly formatted across all developers' workspaces, you should make sure that
`cargo fmt` runs on the files you change/edit/create.

If you use VS Code, we highly recommend the extension [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer). Make sure that you also have _format on save_ enabled. This is what it needs to be in your VS Code settings:

```json
  ...
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },
  ...
```

## Use case

Users with favorable on-chain loan transaction history generate ZK proofs using public inputs demonstrating their creditworthiness without revealing their private financial data.

- **synthetic_data**: Library that provides synthetic credit data generation and model training
- **ezkl**: Main entry point that uses synthetic_data library and integrates with EZKL CLI
- **loan**: CLI interface for the system (in development)

## Workflow

1. Generate synthetic credit data and train a credit scoring model
2. Convert the model to ONNX format compatible with EZKL
3. Use EZKL to create zero-knowledge circuits and proofs
4. Generate a Solidity [verifier](https://github.com/Loan-ZKML/contracts/blob/39f2a849f0a502cd2dc19422fc579e98e03e3f41/src/ZKCreditVerifier.sol#L43) contract  for on-chain verification
5. Allow users to submit proofs to DeFi lending platforms for better loan terms

### Core Components

- **synthetic_data**: Generation of synthetic data for testing and demonstration
- **ezkl**: Zero-knowledge proof generation and verification for machine learning models

## Continuous Integration

The project utilizes GitHub Actions for continuous integration with the following checks:

- **Build**: Ensures all workspace crates compile successfully
- **Test**: Runs the test suite across all crates
- **Check**: Verifies code without producing binaries
- **Clippy**: Enforces Rust's linting rules with no warnings

All these checks must pass for pull requests to be merged, maintaining code quality and project stability.

## Development

A Makefile is provided for convenient development:

```bash
# Build all crates
make

# cargo test --workspace
make test

# cargo check --workspace
make check

make clippy

# See all available commands
make help
```

## Getting Started

### Prerequisites

- Rust toolchain
- Python with PyTorch and ONNX
- EZKL CLI tool installed (version 20.2.0+)

### Installation

```bash
# Clone the repository
git clone https://github.com/loan-zkml/Ezkl-ML.git
cd demo

# Install Python dependencies
pip install torch numpy onnx

# Install EZKL following instructions at
# https://github.com/zkonduit/ezkl
```

### Usage

```bash
# Run the complete pipeline from data generation to ZK proof creation
cd ezkl
cargo run
```

The `ezkl` crate serves as the main entry point for the application, internally using the `synthetic_data` library to:
1. Generate synthetic credit data
2. Train the ML model
3. Save model and sample input files
4. Process everything with the EZKL CLI to create ZK proofs

## Generated Artifacts

This project generates various data files and artifacts during execution, which are not tracked in Git:

- Model files: JSON and ONNX formats of the credit scoring model
- ZK artifacts: Proofs, keys, witness, and compiled circuits
- Solidity contracts: Generated verifier for on-chain verification

See the .gitignore file for the complete list of untracked generated files.

## Documentation

For more detailed information, see [docs/overview.md](docs/overview.md).
