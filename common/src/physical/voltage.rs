use core::{fmt::Display, marker::PhantomData};

use serde::{Deserialize, Serialize};
use thiserror_no_std::Error;

/// Store physical unit value of Voltage.
///
/// ```
/// use common::physical::Voltage;
/// let voltage: Voltage = Voltage::new(3.3f32, 1.8f32)
///     .expect("Failed to get Voltage representation");
/// let underlying_value: f32 = voltage.value();
/// assert_eq!(underlying_value, 1.8f32);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Voltage {
    max: f32,
    value: f32,
    _private: PhantomData<()>,
}

#[derive(Debug, Error)]
pub enum VoltageError {
    /// The Voltage was trying to be created with a value outside of the valid
    /// state space representation. This is due to either a negative value
    /// or too high of a value being ued.
    #[error("Value outside of valid state space representation!")]
    OutOfValidStateSpace,
}

impl Voltage {
    /// Construct a Voltage given a maximum and current value.
    /// Will return `OutOfValidStateSpace` if Voltage is negative or above
    /// maximum.
    pub fn new(max: f32, value: f32) -> Result<Self, VoltageError> {
        if value < 0f32 || value > max {
            return Err(VoltageError::OutOfValidStateSpace);
        }
        Ok(Self {
            max,
            value,
            _private: PhantomData,
        })
    }

    /// Get a copy of the max voltage this instance can represent.
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Get a copy of the current voltage this instance does represent.
    pub fn value(&self) -> f32 {
        self.value
    }
}

impl Display for Voltage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<Voltage: {}/{} V>", self.value(), self.max())
    }
}

impl Into<f32> for Voltage {
    fn into(self) -> f32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let voltage: Result<Voltage, VoltageError> = Voltage::new(5f32, -1f32);
        assert!(voltage.is_err());

        let voltage: Voltage = Voltage::new(5f32, 0f32).expect("Failed to create valid voltage.");
        assert_eq!(voltage.value(), 0f32);
        assert_eq!(voltage.max(), 5f32);

        let voltage: Voltage =
            Voltage::new(5f32, 4.99f32).expect("Failed to create valid voltage.");
        assert_eq!(voltage.value(), 4.99f32);
        assert_eq!(voltage.max(), 5f32);

        let voltage: Voltage =
            Voltage::new(3.3f32, 1.8f32).expect("Failed to create valid voltage.");
        assert_eq!(voltage.value(), 1.8f32);
        assert_eq!(voltage.max(), 3.3f32);

        let voltage: Voltage = Voltage::new(5f32, 5f32).expect("Failed to create valid voltage.");
        assert_eq!(voltage.value(), 5f32);
        assert_eq!(voltage.max(), 5f32);

        let voltage: Result<Voltage, VoltageError> = Voltage::new(5f32, 5.01f32);
        assert!(voltage.is_err());
    }

    #[test]
    fn test_serialization() {
        let voltage = Voltage::new(3.3f32, 1.8f32).expect("Failed to create valid voltage");

        let voltage_ser =
            postcard::to_vec::<Voltage, 64>(&voltage).expect("Failed to serialize Voltage.");
        let voltage_derser =
            postcard::from_bytes::<Voltage>(&voltage_ser).expect("Failed to deserialize Voltage");

        assert_eq!(voltage_derser.value(), 1.8f32);
        assert_eq!(voltage_derser.max(), 3.3f32);
    }
}
