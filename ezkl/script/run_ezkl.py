#!/usr/bin/env python3
import os
import sys
import subprocess
import argparse

def main():
    parser = argparse.ArgumentParser(description="Run EZKL commands")
    parser.add_argument("--working-dir", required=True)
    parser.add_argument("--model-path", required=True)
    parser.add_argument("--srs-path", required=True)
    parser.add_argument("--generate-contract", action="store_true", help="Generate Solidity verifier contract and calldata")
    args = parser.parse_args()

    os.chdir(args.working_dir)
    
    # Ensure EZKL is installed
    try:
        subprocess.run(["ezkl", "--help"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except Exception:
        print("EZKL not found. Please install it with: pip install ezkl")
        sys.exit(1)

    steps = [
        (["ezkl", "gen-settings", "-M", args.model_path, "-O", "settings.json"], "Generating circuit settings..."),
        (["ezkl", "calibrate-settings", "-M", args.model_path, "-D", "input.json", "-O", "settings.json"], "Calibrating settings..."),
        (["ezkl", "compile-circuit", "-M", args.model_path, "--compiled-circuit", "model.compiled", "-S", "settings.json"], "Compiling model to circuit..."),
        (["ezkl", "setup", "-M", "model.compiled", "--pk-path", "pk.key", "--vk-path", "vk.key", "--srs-path", args.srs_path], "Running setup to generate keys..."),
        (["ezkl", "gen-witness", "-D", "input.json", "-M", "model.compiled", "-O", "witness.json"], "Generating witness..."),
        (["ezkl", "prove", "--witness", "witness.json", "--proof-path", "proof.json", "--pk-path", "pk.key", "--compiled-circuit", "model.compiled", "--srs-path", args.srs_path], "Generating proof..."),
        (["ezkl", "verify", "--proof-path", "proof.json", "--vk-path", "vk.key", "--srs-path", args.srs_path], "Verifying proof locally...")
    ]

    for cmd, msg in steps:
        print(msg)
        result = subprocess.run(cmd)
        if result.returncode != 0:
            print(f"Failed step: {' '.join(cmd)}")
            sys.exit(1)
    
    if args.generate_contract:
        contract_step = [
            (["ezkl", "create-evm-verifier", "--vk-path", "vk.key", "--sol-code-path", "Halo2Verifier.sol", "--srs-path", args.srs_path], "Generating Solidity verifier contract..."),
            (["ezkl", "encode-evm-calldata", "--proof-path", "proof.json", "--calldata-path", "calldata.json"], "Generating calldata for on-chain verification...")
        ]
        for cmd, msg in contract_step:
            print(msg)
            result = subprocess.run(cmd)
            if result.returncode != 0:
                print(f"Failed step: {' '.join(cmd)}")
                sys.exit(1)

    print("EZKL processing complete!")

if __name__ == "__main__":
    main()
