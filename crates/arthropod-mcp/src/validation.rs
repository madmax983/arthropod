use anyhow::{Result, anyhow};

/// Validates that an f32 value is finite (not NaN and not Infinity).
pub fn validate_finite(value: f32, field_name: &str) -> Result<f32> {
    if !value.is_finite() {
        return Err(anyhow!(
            "Invalid value for {}: must be finite number",
            field_name
        ));
    }
    Ok(value)
}

/// Validates that an f32 value is finite and within the range [min, max].
pub fn validate_range(value: f32, min: f32, max: f32, field_name: &str) -> Result<f32> {
    validate_finite(value, field_name)?;
    if !(value >= min && value <= max) {
        return Err(anyhow!(
            "Invalid value for {}: must be between {} and {}",
            field_name,
            min,
            max
        ));
    }
    Ok(value)
}
