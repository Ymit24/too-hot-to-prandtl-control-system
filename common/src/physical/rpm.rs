use thiserror_no_std::Error;

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
/// Generic parameter to specify maximum speed.
/// Use Into<f32> to recover the speed value.
///
/// ```
/// use common::physical::rpm::Rpm;
/// let rpm: Rpm<2000> = Rpm::try_from(500.2f32).expect("Failed to get RPM representation.");
/// let underlying_speed: f32 = rpm.into();
/// assert_eq!(underlying_speed, 500.2f32);
/// ```
#[derive(Debug)]
pub struct Rpm<const MAX_SPEED: RpmSpeed> {
    /// The maximum speed this RPM value can represent.
    pub max_speed: u32,

    /// The raw speed value being represented.
    /// Speeds are stored as 100 x speed as u32s to gain
    /// more precision without floating point math.
    /// E.g. 250.5 RPM is stored as 25050u32
    speed_raw: u32,
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

impl<const MAX_SPEED: RpmSpeed> Rpm<MAX_SPEED> {
    /// Private method for creating new RPM types.
    fn new(speed: f32) -> Self {
        Self {
            max_speed: to_rpm_speed(MAX_SPEED as f32).unwrap(),
            speed_raw: (speed * 100f32) as u32,
        }
    }
}

impl<const MAX_SPEED: RpmSpeed> Into<f32> for Rpm<MAX_SPEED> {
    fn into(self) -> f32 {
        from_rpm_speed(self.speed_raw)
    }
}

impl<const MAX_SPEED: RpmSpeed> TryFrom<f32> for Rpm<MAX_SPEED> {
    type Error = RpmError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if to_rpm_speed(value) > to_rpm_speed(MAX_SPEED as f32) || to_rpm_speed(value).is_none() {
            return Err(RpmError::OutOfValidStateSpace);
        }
        Ok(Self::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_f32() {
        let rpm: Result<Rpm<2300>, RpmError> = Rpm::try_from(4000f32);
        assert!(rpm.is_err());

        let rpm: Result<Rpm<2300>, RpmError> = Rpm::try_from(2300f32);
        assert!(rpm.is_ok());

        let rpm: f32 = rpm.unwrap().into();
        assert_eq!(rpm, 2300f32);

        let rpm: Result<Rpm<1200>, RpmError> = Rpm::try_from(-500f32);
        assert!(rpm.is_err());
    }

    #[test]
    fn test_into_f32() {
        let rpm: Rpm<2300> = Rpm::try_from(2000f32).expect("Failed to get RPM representation.");
        let speed: f32 = rpm.into();

        assert_eq!(speed, 2000f32);

        let rpm: Rpm<100> = Rpm::try_from(50.01f32).expect("Failed to get RPM representation.");
        let speed: f32 = rpm.into();
        assert_eq!(speed, 50.01f32);

        let rpm: Rpm<10000> = Rpm::try_from(3250.20f32).expect("Failed to get RPM representation.");
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
}
