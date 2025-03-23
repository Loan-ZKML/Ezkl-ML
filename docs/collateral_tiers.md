# Collateral Tier System Implementation Guide

This document explains how the collateral tier system works within our ZKML credit scoring system, along with implementation details and strategies for testing.

## Overview

DeFi lending protocols typically require substantial collateral (often 150% or more) to secure loans, making borrowing less capital-efficient. Our system introduces a privacy-preserving credit scoring mechanism that allows users to prove their creditworthiness without revealing sensitive financial data, resulting in reduced collateral requirements.

## Collateral Tier Structure

The system implements four distinct collateral tiers based on credit scores:

| Tier | Credit Score Range | Collateral Requirement | Description |
|------|-------------------|------------------------|-------------|
| UNKNOWN | No score/proof | 120% | Default tier for new users with no verifiable credit history |
| LOW | < 0.4 | 100% | Users with minimal credit history or lower scores |
| MEDIUM | 0.4 - 0.7 | 90% | Users with moderate credit history and scores |
| HIGH | > 0.7 | 80% | Users with excellent credit history and scores |

## EZKL Scaling and Test Results

### EZKL Scaling Information

When implementing credit scores in zero-knowledge proofs using EZKL, we must account for scaling factors:

1. **Original ML Model Output**: Scores range from 0.0 to 1.0 (floating-point values)
2. **Human-Readable Scale**: We multiply by 1000 to get scores from 0 to 1000 (integers)
3. **EZKL Circuit Scale**: EZKL applies an additional scaling factor (approximately 67219) for fixed-point arithmetic in the circuit

This means a score goes through the following transformations:
- Original score: e.g., 0.629 (from ML model)
- Human-readable score: 629 (0.629 × 1000)
- EZKL circuit value: 42,280,878 (629 × 67219)

The smart contract then descales the EZKL circuit value back to the human-readable range (0-1000) for tier assignment.

### Actual Test Scores

Our test addresses show the following credit scores:

| Address | Original Score | Human-Readable Score (0-1000) | EZKL Circuit Value | Tier |
|---------|----------------|-------------------------------|-------------------|------|
| 0x2222222222222222222222222222222222222222 | 0.09 | 90 | ~6,049,710 | LOW |
| 0x276ef71c8F12508d187E7D8Fcc2FE6A38a5884B1 | 0.63 | 629 | 42,280,878 | MEDIUM |
| 0x4444444444444444444444444444444444444444 | 0.86 | 860 | ~57,808,340 | HIGH |

These test scores allow us to validate the entire pipeline from ML model to smart contract verification, ensuring each address qualifies for the appropriate collateral tier.

## Ethereum Address Strategy for Testing

### Test Address Convention

For testing and development purposes, we use a consistent set of Ethereum addresses to represent users in different tiers:

- **UNKNOWN Tier**: Any address not included in training data, e.g., `0x1111111111111111111111111111111111111111`
- **LOW Tier**: `0x2222222222222222222222222222222222222222`
- **MEDIUM Tier**: `0x276ef71c8F12508d187E7D8Fcc2FE6A38a5884B1` (prv-key: `0x08c216a5cbe31fd3c8095aae062a101c47c0f6110d738b97c5b1572993a2e665`)
- **HIGH Tier**: `0x4444444444444444444444444444444444444444`

### How Addresses Qualify for Discounts

In this system, addresses qualify for collateral discounts through their credit scores, which are calculated from financial metrics and verified through zero-knowledge proofs. The system does not directly whitelist specific addresses for discounts. Instead:

1. The model evaluates wallet activity metrics to compute a credit score
2. This score determines which collateral tier the address qualifies for
3. The proof verifies this score without revealing the underlying data
4. The smart contract assigns the appropriate tier based on the verified score

The test addresses mentioned (`0x2222...`, `0x3333...`, etc.) are not hardcoded into the production smart contracts. They are simply examples used during development and testing to verify different tiers are working correctly.

### Adding More Qualifying Addresses

To add more addresses that qualify for collateral discounts, you have two options:

#### Option 1: Synthetic Data Approach (for development/testing)

When generating synthetic data, you can include specific addresses with feature values that will result in desired credit scores:

```rust
// Example feature values for addresses (pseudo-code)
// Format: [tx_count, wallet_age, avg_balance, repayment_history]

// Add specific real addresses to the model training data
// LOW tier address (score < 0.4)
data.add_record("0x2222222222222222222222222222222222222222", [0.1, 0.2, 0.1, 0.0]);

// MEDIUM tier address (score between 0.4 and 0.7)
data.add_record("0x276ef71c8F12508d187E7D8Fcc2FE6A38a5884B1", [0.5, 0.4, 0.5, 1.0]);

// HIGH tier address (score > 0.7)
data.add_record("0x4444444444444444444444444444444444444444", [0.9, 0.8, 0.7, 1.0]);

// Add additional addresses that should qualify for specific tiers
// Another HIGH tier address
data.add_record("0x5555555555555555555555555555555555555555", [0.85, 0.9, 0.8, 1.0]);

// You can add as many addresses as needed for testing different scenarios
```

This approach is primarily useful for testing and development. In the `synthetic_data` crate, you would modify the `generate_synthetic_data` function to include these specific addresses alongside the randomly generated data.

#### Option 2: Production Approach (for real-world use)

In a production environment, addresses qualify for discounts based on their actual on-chain activity:

1. **Data Collection**: Collect real on-chain data for the address (transaction count, wallet age, etc.)
2. **Score Calculation**: Use the trained model to calculate the address's credit score
3. **Proof Generation**: Generate a zero-knowledge proof of this calculation
4. **On-chain Verification**: Submit the proof to the smart contract, which verifies it and assigns the appropriate tier

To add support for more addresses in production:

1. Expand the data collection pipeline to gather metrics for more addresses
2. Ensure the model generalizes well to different address activity patterns
3. Provide tools for users to generate proofs of their own credit scores
4. Update the smart contract if needed to handle specific edge cases

```rust
// Example code for generating proof for a new address (pseudo-code)
fn generate_proof_for_address(address: &str, model: &CreditScoreModel) -> Result<Proof> {
    // 1. Collect real metrics for this address
    let metrics = collect_on_chain_metrics(address)?;
    
    // 2. Normalize the metrics to match model expectations
    let normalized_metrics = normalize_metrics(metrics);
    
    // 3. Calculate score using the model
    let score = model.predict(&normalized_metrics);
    
    // 4. Generate ZK proof
    let proof = generate_zk_proof(address, &normalized_metrics, score, model)?;
    
    Ok(proof)
}
```

Remember that in production, you wouldn't directly assign tiers to addresses in the code. Instead, addresses earn their tier by proving their credit score.

### Implementation in Smart Contracts

The on-chain collateral calculator would implement logic similar to:

```solidity
contract CollateralCalculator is ICollateralCalculator {
    mapping(address => CreditTier) private addressTiers;
    mapping(CreditTier => uint256) private tierCollateralPercentages;
    
    constructor() {
        // Set default collateral percentages for each tier (in basis points)
        tierCollateralPercentages[CreditTier.UNKNOWN] = 12000; // 120%
        tierCollateralPercentages[CreditTier.LOW] = 10000;     // 100%
        tierCollateralPercentages[CreditTier.MEDIUM] = 9000;   //  90%
        tierCollateralPercentages[CreditTier.HIGH] = 8000;     //  80%
    }
    
    function getCollateralRequirement(address _borrower, uint256 _loanAmount) 
        external 
        view 
        override 
        returns (CollateralRequirement memory) 
    {
        // Get the borrower's tier, defaulting to UNKNOWN
        CreditTier tier = addressTiers[_borrower];
        
        // Get the collateral percentage for this tier
        uint256 percentage = tierCollateralPercentages[tier];
        
        // Calculate required collateral amount
        uint256 requiredAmount = (_loanAmount * percentage) / 10000;
        
        return CollateralRequirement({
            requiredPercentage: percentage,
            requiredAmount: requiredAmount,
            tier: tier
        });
    }
    
    function updateCreditScore(
        address _borrower, 
        uint256 _creditScore, 
        bool _proofValidated
    ) external override returns (CreditTier) {
        require(_proofValidated, "Invalid proof");
        
        // Determine tier based on credit score
        // Note: _creditScore is the descaled value (0-1000 range)
        // It has been converted from the EZKL circuit value using:
        // descaled_score = ezkl_raw_score / 67219
        CreditTier newTier;
        if (_creditScore < 400) {         // < 0.4
            newTier = CreditTier.LOW;
        } else if (_creditScore < 700) {  // 0.4 - 0.7
            newTier = CreditTier.MEDIUM;
        } else {                          // > 0.7
            newTier = CreditTier.HIGH;
        }
        
        // Update the borrower's tier
        addressTiers[_borrower] = newTier;
        
        return newTier;
    }
}
```

## Testing Strategy

### End-to-End Test Flow

1. **Generate Synthetic Data**: Create synthetic data that includes test addresses with features corresponding to their desired tiers.

2. **Train Model**: Train the model on this data to ensure the test addresses receive appropriate credit scores.

3. **Generate Proofs**: Create ZK proofs for each test address:
   - For the HIGH tier address, create a proof of a credit score > 0.7
   - For the MEDIUM tier address, create a proof of a credit score between 0.4 and 0.7
   - For the HIGH tier address (0x4444...), create a proof of a credit score > 0.7 (actual: 0.86)
   - For the MEDIUM tier address (0x276e...), create a proof of a credit score between 0.4 and 0.7 (actual: 0.63)
   - For the LOW tier address (0x2222...), create a proof of a credit score < 0.4 (actual: 0.09)
   - For the UNKNOWN tier test, use an address not in the training data

### Unknown User Testing

For testing the UNKNOWN tier (users without proof or history):

1. Use an Ethereum address not included in the training data
2. Attempt to request a loan without providing a ZK proof
3. Verify that the contract requires 120% collateral

### Key Test Cases

- **New user with no proof**: Should receive 120% collateral requirement
- **User with LOW score proof**: Should receive 100% collateral requirement
- **User with MEDIUM score proof**: Should receive 90% collateral requirement
- **User with HIGH score proof**: Should receive 80% collateral requirement
- **User with expired/invalid proof**: Should revert to 120% collateral requirement
- **Loan request with insufficient collateral**: Should be rejected

## Implementation Considerations

- **Proof Expiration**: Consider implementing a time-based expiration for proofs (e.g., 30 days) to ensure users maintain good standing.
- **Tier Transitions**: Monitor and handle cases where users move between tiers.
- **Interest Rate Variation**: Consider varying interest rates based on tiers in addition to collateral requirements.
- **Governance**: Consider implementing a governance mechanism for adjusting tier thresholds and collateral percentages.

## Integration with Frontend/CLI

The frontend or CLI application should follow these steps to interact with the system:

1. Generate credit score proof using the user's Ethereum address and financial data
2. Submit the proof to the smart contract
3. Request collateral requirement preview based on the verified proof
4. Submit loan request with appropriate collateral