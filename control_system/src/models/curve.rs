use std::marker::PhantomData;
use thiserror::Error;

/// This represents a curve mapping some `X` type to some `Y` type.
/// This will be used to define activation curves in the various control systems.
/// This supports unit based curves. (e.g. RPM vs degC)
///
/// Curves can't be empty.
pub struct Curve<X: Into<f32>, Y: Into<f32>> {
    /// Control points for interpolation.
    points: Vec<(X, Y)>,
    _marker: PhantomData<()>,
}

#[derive(Error, Debug)]
pub enum CurveError {
    #[error("Curves can't be empty.")]
    Empty,
}

impl<X: Clone + Copy + Into<f32>, Y: Clone + Copy + Into<f32> + TryFrom<f32>> Curve<X, Y> {
    /// Create a new curve from a set of control points.
    /// This curve must not be empty.
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
    pub fn lookup(&self, x: X) -> Option<Y> {
        let xy1 = self.find_last_point_before_x(x.clone()).unwrap();
        let xy2 = self.find_first_point_after_x(x.clone()).unwrap();

        let x1: f32 = xy1.0.into();
        let x2: f32 = xy2.0.into();

        let y1: f32 = xy1.1.into();
        let y2: f32 = xy2.1.into();

        if x1 == x2 {
            return Some(xy1.1);
        }

        match Y::try_from(y1 + (y2 - y1) * ((x.into() - x1) / (x2 - x1))) {
            Err(_) => None,
            Ok(value) => Some(value),
        }
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
            .filter(|xi| xi.0.into() <= x.into())
            .collect::<Vec<_>>();
        point_xs.sort_by(|x, y| x.0.into().partial_cmp(&y.0.into()).unwrap());
        point_xs.into_iter().last().or(self
            .points
            .clone()
            .into_iter()
            .min_by(|x, y| x.0.into().partial_cmp(&y.0.into()).unwrap()))
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
            .filter(|xi| x.into() <= xi.0.into())
            .collect::<Vec<_>>();
        point_xs.sort_by(|x, y| x.0.into().partial_cmp(&y.0.into()).unwrap());
        point_xs.into_iter().rev().last().or(self
            .points
            .clone()
            .into_iter()
            .max_by(|x, y| x.0.into().partial_cmp(&y.0.into()).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(curve.lookup(-3f32).expect("Failed to lookup value"), 0f32);
        assert_eq!(curve.lookup(0f32).expect("Failed to lookup value"), 0f32);
        assert_eq!(curve.lookup(1f32).expect("Failed to lookup value"), 1f32);
        assert_eq!(curve.lookup(3f32).expect("Failed to lookup value"), 3f32);
        assert_eq!(curve.lookup(10f32).expect("Failed to lookup value"), 10f32);
        assert_eq!(curve.lookup(100f32).expect("Failed to lookup value"), 10f32);
    }

    #[derive(Copy, Clone, PartialEq, PartialOrd)]
    struct TempC {
        value: f32,
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

        assert_eq!(
            curve.lookup(0f32.into()).expect("Failed to lookup value"),
            10f32
        );
        assert_eq!(
            curve.lookup(30f32.into()).expect("Failed to lookup value"),
            10f32
        );
        assert_eq!(
            curve.lookup(45f32.into()).expect("Failed to lookup value"),
            30f32
        );
        assert_eq!(
            curve.lookup(60f32.into()).expect("Failed to lookup value"),
            50f32
        );
        assert_eq!(
            curve.lookup(70f32.into()).expect("Failed to lookup value"),
            75f32
        );
        assert_eq!(
            curve.lookup(80f32.into()).expect("Failed to lookup value"),
            100f32
        );
    }
}
