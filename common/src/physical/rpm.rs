use core::{fmt::Display, marker::PhantomData, ops::Sub};

use serde::{Deserialize, Serialize};
use thiserror_no_std::Error;

use super::Percentage;

/// Represent the underlying storage type for the RpmSpeed
type RpmSpeed = u32;

/// Convert a nice f32 representation into
/// the underlying storage type.
fn to_rpm_speed(raw: f32) -> Option<RpmSpeed> {
    if raw.is_sign_negative() {
        return None;
    }
    Some((raw * 100f32) as RpmSpeed)
}

/// Convert a `RpmSpeed` into a nice f32
/// representation.
fn from_rpm_speed(speed: RpmSpeed) -> f32 {
    (speed as f32 / 100f32) as f32
}

/// Store physical unit value of Rotations Per Minute (RPM).
///
/// ```
/// use common::physical::Rpm;
/// let rpm: Rpm = Rpm::new(2000f32, 500.2f32).expect("Failed to get RPM representation.");
/// let underlying_speed: f32 = rpm.speed();
/// assert_eq!(underlying_speed, 500.2f32);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Rpm {
    /// The maximum speed this RPM value can represent.
    max_speed_raw: u32,

    /// The raw speed value being represented.
    /// Speeds are stored as 100 x speed as u32s to gain
    /// more precision without floating point math.
    /// E.g. 250.5 RPM is stored as 25050u32
    speed_raw: u32,

    /// Make sure this can't be constructed with struct literals.
    /// This ensures that state space representation boundaries aren't
    /// circumvented.
    _private: PhantomData<()>,
}

/// Represents errors in creating or using the RPM type.
#[derive(Debug, Error)]
pub enum RpmError {
    /// The RPM was trying to be created with a value outside of the valid
    /// state space representation. This is due to either a negative
    /// value or too high of value being used.
    #[error("Value outside of valid state space representation!")]
    OutOfValidStateSpace,
}

impl Rpm {
    /// Construct a RPM given a max and current speed.
    /// Will return `OutOfValidStateSpace` if RPM is negative or above
    /// maximum.
    pub fn new(max_speed: f32, speed: f32) -> Result<Self, RpmError> {
        let max_speed = match to_rpm_speed(max_speed) {
            None => return Err(RpmError::OutOfValidStateSpace),
            Some(rpm_speed) => rpm_speed,
        };
        let current_speed = match to_rpm_speed(speed) {
            None => return Err(RpmError::OutOfValidStateSpace),
            Some(rpm_speed) => rpm_speed,
        };

        if current_speed > max_speed {
            return Err(RpmError::OutOfValidStateSpace);
        }
        Ok(Self {
            max_speed_raw: max_speed,
            speed_raw: current_speed,
            _private: PhantomData,
        })
    }

    /// Get the maximum speed that this RPM can represent.
    /// Converts from the underlying storage type.
    pub fn max_speed(&self) -> f32 {
        from_rpm_speed(self.max_speed_raw)
    }

    /// Get the current speed that this RPM does represent.
    /// Converts from the underlying storage type.
    pub fn speed(&self) -> f32 {
        from_rpm_speed(self.speed_raw)
    }

    /// Subtract another RPM's value from this RPM. Keeps this RPM's max speed.
    pub fn sub(&self, rhs: Self) -> Result<Self, RpmError> {
        Self::new(
            self.max_speed(),
            from_rpm_speed(self.speed_raw) - from_rpm_speed(rhs.speed_raw),
        )
    }

    /// Convert `RPM` into `Percentage`.
    /// ```
    /// use crate::common::physical::{Rpm,Percentage};
    /// let rpm = Rpm::new(1000f32, 500f32).expect("Failed to generate RPM.");
    /// let percentage = rpm.into_percentage();
    /// assert_eq!(percentage, Percentage::try_from(50f32).expect("Failed to generate Percentage"));
    /// ```
    pub fn into_percentage(&self) -> Percentage {
        Percentage::try_from((self.speed() / self.max_speed()) * 100f32)
            .expect("Failed to generate Percentage.")
    }
}

impl Display for Rpm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<Rpm: {}/{} RPM>", self.speed(), self.max_speed())
    }
}

impl Into<f32> for Rpm {
    fn into(self) -> f32 {
        from_rpm_speed(self.speed_raw)
    }
}

impl TryFrom<f32> for Rpm {
    type Error = RpmError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_sign_negative() {
            return Err(RpmError::OutOfValidStateSpace);
        }
        Rpm::new(1f32, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let rpm: Result<Rpm, RpmError> = Rpm::new(2300f32, 4000f32);
        assert!(rpm.is_err());

        let rpm: Result<Rpm, RpmError> = Rpm::new(2300f32, 2300f32);
        assert!(rpm.is_ok());

        let rpm: f32 = rpm.unwrap().into();
        assert_eq!(rpm, 2300f32);

        let rpm: Result<Rpm, RpmError> = Rpm::new(2300f32, -500f32);
        assert!(rpm.is_err());
    }

    #[test]
    fn test_into_f32() {
        let rpm = Rpm::new(2300f32, 2000f32).expect("Failed to get RPM representation.");
        let speed: f32 = rpm.into();

        assert_eq!(speed, 2000f32);

        let rpm = Rpm::new(100f32, 50.01f32).expect("Failed to get RPM representation.");
        let speed: f32 = rpm.into();
        assert_eq!(speed, 50.01f32);

        let rpm = Rpm::new(5000f32, 3250.20f32).expect("Failed to get RPM representation.");
        let speed: f32 = rpm.into();
        assert_eq!(speed, 3250.20f32);
    }

    #[test]
    fn test_to_rpm_speed() {
        assert_eq!(to_rpm_speed(-100.23f32), None);
        assert_eq!(to_rpm_speed(-1f32), None);
        assert_eq!(to_rpm_speed(0f32), Some(0));
        assert_eq!(to_rpm_speed(100f32), Some(100_00));
        assert_eq!(to_rpm_speed(100.50f32), Some(100_50));
        assert_eq!(to_rpm_speed(1352.22f32), Some(1352_22));
        assert_eq!(to_rpm_speed(2300f32), Some(2300_00));
    }

    #[test]
    fn test_from_rpm_speed() {
        assert_eq!(from_rpm_speed(0_0), 0f32);
        assert_eq!(from_rpm_speed(300_80), 300.8f32);
        assert_eq!(from_rpm_speed(100_00), 100f32);
        assert_eq!(from_rpm_speed(100_53), 100.53f32);
    }

    #[test]
    fn test_rpm_serialization() {
        let rpm = Rpm::new(2000f32, 1000.55f32).expect("Failed to get RPM representation");

        let rpm_ser = postcard::to_vec::<Rpm, 64>(&rpm).expect("Failed to serialize RPM");
        let rpm_deser = postcard::from_bytes::<Rpm>(&rpm_ser).expect("Failed to deserialize RPM");

        assert_eq!(
            rpm_deser.max_speed_raw,
            to_rpm_speed(2000f32).expect("Failed to convert to RPM format.")
        );
        assert_eq!(
            rpm_deser.speed_raw,
            to_rpm_speed(1000.55f32).expect("Failed to convert to RPM format.")
        );
    }

    #[test]
    fn test_rpm_sub_working_cases() {
        let rpm1 = Rpm::new(1000f32, 500f32).expect("Failed to get RPM");
        let rpm2 = rpm1.clone();
        let rpm3 = Rpm::new(1000f32, 250f32).expect("Failed to get RPM");

        let new_rpm = rpm1.sub(rpm3);
        assert!(new_rpm.is_ok());

        let new_rpm = new_rpm.expect("Failed to subtract RPMs!");
        assert_eq!(new_rpm.speed(), 250f32);

        let new_rpm = rpm1.sub(rpm2);
        assert!(new_rpm.is_ok());

        let new_rpm = new_rpm.expect("Failed to subtract RPMs!");
        assert_eq!(new_rpm.speed(), 0f32);
    }

    #[test]
    fn test_rpm_sub_failing_cases() {
        let rpm1 = Rpm::new(1000f32, 500f32).expect("Failed to get RPM");
        let rpm2 = Rpm::new(3000f32, 2500f32).expect("Failed to get RPM");

        let new_rpm = rpm1.sub(rpm2);
        assert!(new_rpm.is_err());
    }
}
