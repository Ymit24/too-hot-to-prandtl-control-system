use core::{fmt::Display, marker::PhantomData};
use fixed::{
    types::{extra::U3, I13F3},
    FixedI16,
};
use serde::{Deserialize, Serialize};
use thiserror_no_std::Error;

/// Type alias for how the percentage value is actually stored.
pub type PercentageValue = I13F3;

/// Represents a 0-100% value. Stores with two decimal places of precision
/// using quarter percent steps.
///
/// ```
/// use common::physical::Percentage;
/// let raw: f32 = 50f32;
/// let percent = Percentage::try_from(raw).expect("Failed to get Percentage representation");
/// assert_eq!(percent.value(), raw);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Percentage {
    value: PercentageValue,
}

/// Represents errors in creating or using the `Percentage` type.
#[derive(Debug, Error)]
pub enum PercentageError {
    /// The `Percentage` was trying to be created with a value outside of the valid
    /// state space representation. This is due to either a negative value
    /// or too high of a value being ued.
    #[error("Value outside of valid state space representation!")]
    OutOfValidStateSpace,
}

impl Percentage {
    /// Get the underlying percentage value.
    pub fn value(&self) -> PercentageValue {
        self.value.clone()
    }

    /// Subtract a percentage from this percentage.
    pub fn sub(&self, rhs: Self) -> Result<Self, PercentageError> {
        Percentage::try_from((self.value() - rhs.value()).to_num::<f32>())
    }
}

impl TryFrom<f32> for Percentage {
    type Error = PercentageError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value < 0f32 || value > 100f32 {
            return Err(PercentageError::OutOfValidStateSpace);
        }
        Ok(Self {
            value: PercentageValue::from_num(value),
        })
    }
}

impl Into<f32> for Percentage {
    fn into(self) -> f32 {
        self.value.into()
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<Percentage: {}%>", self.value)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_creation_with_quarter_steps() {
        let percent = Percentage::try_from(-5f32);
        assert!(percent.is_err());

        for i in 0..400 {
            let raw: f32 = (i as f32) / 4f32;
            let percent =
                Percentage::try_from(raw).expect("Failed to get valid Percentage representation.");
            assert_eq!(percent.value, raw);
        }

        let percent = Percentage::try_from(105f32);
        assert!(percent.is_err());
    }

    #[test]
    fn test_sub_working_cases() {
        let perc1 = Percentage::try_from(50f32).expect("Failed to get Percentage.");
        let perc2 = perc1.clone();
        let perc3 = Percentage::try_from(25f32).expect("Failed to get Percentage.");

        let new_perc = perc1.sub(perc2).expect("Failed to subtract Percentages.");
        assert_eq!(new_perc.value(), 0);

        let new_perc = perc1.sub(perc3).expect("Failed to subtract Percentages.");
        assert_eq!(new_perc.value(), 25f32);
    }

    #[test]
    fn test_sub_failing_cases() {
        let perc1 = Percentage::try_from(50f32).expect("Failed to get Percentage.");
        let perc2 = Percentage::try_from(75f32).expect("Failed to get Percentage.");

        let new_perc = perc1.sub(perc2);
        assert!(new_perc.is_err());
    }
}
