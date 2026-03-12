use super::super::models::*;

pub(crate) fn compute_centroid(vectors: &[Vec<f32>]) -> Vec<f32> {
    let dimensions = vectors[0].len();
    let mut accum = vec![0.0_f32; dimensions];
    for vector in vectors {
        for (index, value) in vector.iter().enumerate() {
            accum[index] += *value;
        }
    }
    let divisor = vectors.len() as f32;
    accum.iter_mut().for_each(|value| *value /= divisor);
    accum
}

pub(crate) fn cosine_similarity(
    left: &[f32],
    right: &[f32],
) -> Result<f32, DataLayerM5VectorIntegrationError> {
    if left.len() != right.len() {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
            expected: left.len(),
            found: right.len(),
        });
    }
    let (dot, left_norm, right_norm) = norm_totals(left, right);
    if left_norm <= f64::EPSILON || right_norm <= f64::EPSILON {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorValue(
            "zero_norm_vector",
        ));
    }
    Ok((dot / (left_norm.sqrt() * right_norm.sqrt())) as f32)
}

fn norm_totals(left: &[f32], right: &[f32]) -> (f64, f64, f64) {
    left.iter().zip(right.iter()).fold(
        (0.0_f64, 0.0_f64, 0.0_f64),
        |(dot, left_norm, right_norm), (left_value, right_value)| {
            (
                dot + *left_value as f64 * *right_value as f64,
                left_norm + (*left_value as f64).powi(2),
                right_norm + (*right_value as f64).powi(2),
            )
        },
    )
}
