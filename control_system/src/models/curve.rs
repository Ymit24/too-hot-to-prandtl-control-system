use std::marker::PhantomData;
use std::ops::{Add, Div, Mul, Sub};
use thiserror::Error;

/// This represents a curve mapping some `X` type to some `Y` type.
/// This will be used to define activation curves in the various control systems.
/// This supports unit based curves. (e.g. RPM vs degC)
///
/// Curves can't be empty.
pub struct Curve<X: PartialOrd + Clone, Y: Clone> {
    points: Vec<(X, Y)>,
    _marker: PhantomData<()>,
}

#[derive(Error, Debug)]
pub enum CurveError {
    #[error("Curves can't be empty.")]
    Empty,
}

/// This trait makes sure that a type can be scaled by a f32.
pub trait LinearInterp {
    /// Scale the underlying value by `x`.
    /// e.g. `10f32.scale(0.5f32) == 5f32`
    /// or `100RPM.scale(0.1f32) == 10RPM`
    fn scale(self, x: f32) -> Self;
}

impl<
        X: PartialOrd + Clone + Copy + Add<Output = X> + Sub<Output = X> + Div<Output = X> + Into<f32>,
        Y: Copy
            + Clone
            + Sub<Output = Y>
            + Add<Output = Y>
            + Mul<Output = Y>
            + LinearInterp
            + PartialEq,
    > Curve<X, Y>
{
    pub fn new(points: Vec<(X, Y)>) -> Result<Self, CurveError> {
        if points.len() == 0 {
            return Err(CurveError::Empty);
        }
        Ok(Self {
            points,
            _marker: PhantomData,
        })
    }

    /// Perform a linear interpolation to determine the value for a given x.
    /// This will clamp to the lowest value if `x` is lower than the lowest control point.
    /// This will clamp to the highest value if `x` is higher than the highest control point.
    pub fn lookup(&self, x: X) -> Y {
        let xy1 = self.find_last_point_before_x(x.clone()).unwrap();
        let xy2 = self.find_first_point_after_x(x.clone()).unwrap();

        if xy1.0 == xy2.0 {
            return xy1.1;
        }

        xy1.1 + (xy2.1 - xy1.1).scale(((x - xy1.0) / (xy2.0 - xy1.0)).into())
    }

    /// Find the last point before `x` or the earliest point.
    /// E.g. for the curve containing [(0,0), (10,1)]:
    ///     find_last_point_before_x(-3) -> (0,0)
    ///     find_last_point_before_x(3) -> (0,0)
    ///     find_last_point_before_x(12) -> (10,1)
    fn find_last_point_before_x(&self, x: X) -> Option<(X, Y)> {
        let mut point_xs = self
            .points
            .clone()
            .into_iter()
            .filter(|xi| xi.0 <= x)
            .collect::<Vec<_>>();
        point_xs.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
        point_xs.into_iter().last().or(self
            .points
            .clone()
            .into_iter()
            .min_by(|x, y| x.0.partial_cmp(&y.0).unwrap()))
    }

    /// Find the first point after `x` or the latest point.
    /// E.g. for the curve containing [(0,0), (10,1)]:
    ///     find_first_point_after_x(-3) -> (0,0)
    ///     find_first_point_after_x(3) -> (10,1)
    ///     find_first_point_after_x(12) -> (10,1)
    fn find_first_point_after_x(&self, x: X) -> Option<(X, Y)> {
        let mut point_xs = self
            .points
            .clone()
            .into_iter()
            .filter(|xi| x <= xi.0)
            .collect::<Vec<_>>();
        point_xs.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
        point_xs.into_iter().rev().last().or(self
            .points
            .clone()
            .into_iter()
            .max_by(|x, y| x.0.partial_cmp(&y.0).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl LinearInterp for f32 {
        fn scale(self, x: f32) -> Self {
            self * x
        }
    }

    #[test]
    fn test_cant_construct_empty_curve() {
        let curve: Result<Curve<f32, f32>, CurveError> = Curve::new(vec![]);
        assert!(curve.is_err());
    }

    #[test]
    fn test_find_last_point_before_x() {
        let points = vec![(0i16, 0f32), (3, 3f32), (10, 10f32)];
        let curve = Curve::new(points).unwrap();

        assert_eq!(curve.find_last_point_before_x(-3), Some((0i16, 0f32)));
        assert_eq!(curve.find_last_point_before_x(0), Some((0i16, 0f32)));
        assert_eq!(curve.find_last_point_before_x(1), Some((0i16, 0f32)));
        assert_eq!(curve.find_last_point_before_x(3), Some((3i16, 3f32)));
        assert_eq!(curve.find_last_point_before_x(4), Some((3i16, 3f32)));
        assert_eq!(curve.find_last_point_before_x(10), Some((10i16, 10f32)));
        assert_eq!(curve.find_last_point_before_x(100), Some((10i16, 10f32)));
    }

    #[test]
    fn test_find_first_point_after_x() {
        let points = vec![(0i16, 0f32), (3, 3f32), (10, 10f32)];
        let curve = Curve::new(points).unwrap();

        assert_eq!(curve.find_first_point_after_x(-3), Some((0i16, 0f32)));
        assert_eq!(curve.find_first_point_after_x(0), Some((0i16, 0f32)));
        assert_eq!(curve.find_first_point_after_x(1), Some((3i16, 3f32)));
        assert_eq!(curve.find_first_point_after_x(3), Some((3i16, 3f32)));
        assert_eq!(curve.find_first_point_after_x(4), Some((10i16, 10f32)));
        assert_eq!(curve.find_first_point_after_x(10), Some((10i16, 10f32)));
        assert_eq!(curve.find_first_point_after_x(100), Some((10i16, 10f32)));
    }

    #[test]
    fn test_lookup() {
        let points = vec![(0f32, 0f32), (3f32, 3f32), (10f32, 10f32)];
        let curve = Curve::new(points).unwrap();

        assert_eq!(curve.lookup(-3f32), 0f32);
        assert_eq!(curve.lookup(0f32), 0f32);
        assert_eq!(curve.lookup(1f32), 1f32);
        assert_eq!(curve.lookup(3f32), 3f32);
        assert_eq!(curve.lookup(10f32), 10f32);
        assert_eq!(curve.lookup(100f32), 10f32);
    }

    #[derive(Copy, Clone, PartialEq, PartialOrd)]
    struct TempC {
        value: f32,
    }

    impl Sub for TempC {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self {
                value: self.value - rhs.value,
            }
        }
    }

    impl Add for TempC {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self {
                value: self.value + rhs.value,
            }
        }
    }

    impl Div for TempC {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self {
                value: self.value / rhs.value,
            }
        }
    }

    impl LinearInterp for TempC {
        fn scale(self, x: f32) -> Self {
            Self {
                value: self.value * x,
            }
        }
    }

    impl Into<f32> for TempC {
        fn into(self) -> f32 {
            self.value
        }
    }

    impl From<f32> for TempC {
        fn from(value: f32) -> Self {
            Self { value }
        }
    }

    #[test]
    fn test_with_physical_unit() {
        let points: Vec<(TempC, f32)> = vec![
            (0f32, 10f32),
            (30f32, 10f32),
            (60f32, 50f32),
            (80f32, 100f32),
        ]
        .into_iter()
        .map(|x| (x.0.into(), x.1))
        .collect();

        let curve = Curve::new(points).unwrap();

        assert_eq!(curve.lookup(0f32.into()), 10f32);
        assert_eq!(curve.lookup(30f32.into()), 10f32);
        assert_eq!(curve.lookup(45f32.into()), 30f32);
        assert_eq!(curve.lookup(60f32.into()), 50f32);
        assert_eq!(curve.lookup(70f32.into()), 75f32);
        assert_eq!(curve.lookup(80f32.into()), 100f32);
    }
}
