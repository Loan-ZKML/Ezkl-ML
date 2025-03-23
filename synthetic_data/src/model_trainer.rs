use crate::synthetic_data::CreditData;
use anyhow::Result;
use ndarray::{s, Array1, Array2};
use serde_json::json;
use std::f32::consts::E;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct CreditScoreModel {
    coefficients: Vec<f32>,
    intercept: f32,
}

impl CreditScoreModel {
    pub fn train(data: &CreditData) -> Result<Self> {
        // Convert data to ndarray format
        let features = Array2::from_shape_vec(
            (data.features.len(), data.features[0].len()),
            data.features.iter().flatten().cloned().collect(),
        )?;

        let targets = Array1::from_vec(data.scores.clone());

        // Add a column of ones to features for the intercept
        let n_samples = features.shape()[0];
        let mut design_matrix = Array2::ones((n_samples, features.shape()[1] + 1));
        let mut slice = design_matrix.slice_mut(s![.., 1..]);
        slice.assign(&features);

        // Calculate coefficients using the normal equation: β = (X^T X)^(-1) X^T y
        // First, calculate X^T X
        let xt_x = design_matrix.t().dot(&design_matrix);

        // Calculate the pseudo-inverse using a simple approach (not robust for all cases)
        // For a production system, use a more robust solution
        let xt_y = design_matrix
            .t()
            .dot(&targets.into_shape((n_samples, 1)).unwrap());

        // Solve the linear system using a simple Gaussian elimination
        // This is a simplified approach - in production, use a library with QR decomposition
        let mut coeffs = vec![0.0; design_matrix.shape()[1]];

        // Very simplified solver for demo purposes
        // In real applications, use ndarray-linalg or another proper solver
        for i in 0..design_matrix.shape()[1] {
            let mut sum = xt_y[[i, 0]];
            for j in 0..design_matrix.shape()[1] {
                if i != j {
                    sum -= xt_x[[i, j]] * coeffs[j];
                }
            }
            coeffs[i] = sum / xt_x[[i, i]];
        }

        // Extract intercept and feature coefficients
        let intercept = coeffs[0];
        let feature_coeffs = coeffs[1..].to_vec();

        Ok(Self {
            coefficients: feature_coeffs,
            intercept,
        })
    }

    pub fn predict(&self, features: &Array2<f32>) -> Array1<f32> {
        let n_samples = features.shape()[0];
        let mut predictions = Array1::zeros(n_samples);

        for i in 0..n_samples {
            let mut pred = self.intercept;
            for j in 0..self.coefficients.len() {
                pred += features[[i, j]] * self.coefficients[j];
            }

            // Sigmoid function to convert to probability
            pred = if pred < -20.0 {
                0.0
            } else if pred > 20.0 {
                1.0
            } else {
                1.0 / (1.0 + E.powf(-pred))
            };
            predictions[i] = pred;
        }

        predictions
    }

    // Export model to a simple JSON format instead of ONNX
    pub fn export_to_json(&self, path: &Path) -> Result<()> {
        let model_json = json!({
            "intercept": self.intercept,
            "coefficients": self.coefficients,
            "num_features": self.coefficients.len()
        });

        let mut file = File::create(path)?;
        file.write_all(serde_json::to_string_pretty(&model_json)?.as_bytes())?;

        Ok(())
    }

    // Save a sample input for testing
    pub fn save_sample_input(&self, data: &CreditData, path: &str) -> Result<()> {
        // Take first sample from data
        let sample_input = &data.features[0];
        let sample_output = self.predict(
            &Array2::from_shape_vec((1, sample_input.len()), sample_input.clone()).unwrap(),
        );

        let input_json = json!({
            "model": {
                "intercept": self.intercept,
                "coefficients": self.coefficients
            },
            "sample": {
                "features": sample_input,
                "predicted_score": sample_output[0]
            },
            "manual_calculation": {
                "intercept_term": self.intercept,
                "coefficient_terms": self.coefficients.iter().enumerate()
                    .map(|(i, &c)| json!({
                        "feature_index": i,
                        "coefficient": c,
                        "feature_value": sample_input[i],
                        "product": c * sample_input[i]
                    }))
                    .collect::<Vec<_>>(),
                "sum_before_clamp": self.intercept + self.coefficients.iter().enumerate()
                    .map(|(i, &c)| c * sample_input[i])
                    .sum::<f32>(),
                "final_clamped_output": sample_output[0]
            }
        });

        let mut file = File::create(path)?;
        file.write_all(serde_json::to_string_pretty(&input_json)?.as_bytes())?;

        Ok(())
    }

    // Helper method to get model parameters
    pub fn get_model_params(&self) -> (f32, &[f32]) {
        (self.intercept, &self.coefficients)
    }
}
