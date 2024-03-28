use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Add, Deref, Div, Mul, Sub};
use thiserror::Error;

/// This represents a curve mapping some `X` type to some `Y` type.
/// This will be used to define activation curves in the various control systems.
/// This supports unit based curves. (e.g. RPM vs degC)
///
/// Curves can't be empty.
pub struct Curve<X: PartialOrd + Ord + Hash + Clone, Y: Clone> {
    points: HashMap<X, Y>,
    _marker: PhantomData<()>,
}

#[derive(Error, Debug)]
pub enum CurveError {
    #[error("Curves can't be empty.")]
    Empty,
}

pub trait LinearInterp {
    fn norm_scale(self, x: f32) -> Self;
}

impl<
        X: PartialOrd
            + Ord
            + Hash
            + Clone
            + Copy
            + Add<Output = X>
            + Sub<Output = X>
            + Div<Output = X>
            + Into<f32>,
        Y: Copy + Clone + Sub<Output = Y> + Add<Output = Y> + Mul<Output = Y> + LinearInterp + PartialEq,
    > Curve<X, Y>
{
    pub fn new(points: HashMap<X, Y>) -> Result<Self, CurveError> {
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
        let last_point_below_x = self.find_last_point_before_x(x.clone()).unwrap();
        let first_point_after_x = self.find_first_point_after_x(x.clone()).unwrap();

        let last_point_below_x_y = self.points.get(&last_point_below_x).unwrap().clone();
        let first_point_after_x_y = self.points.get(&first_point_after_x).unwrap().clone();

        if last_point_below_x_y == first_point_after_x_y {
            return last_point_below_x_y;
        }

        last_point_below_x_y
            + (first_point_after_x_y - last_point_below_x_y).norm_scale(
                ((x - last_point_below_x) / (first_point_after_x - last_point_below_x)).into(),
            )
    }

    /// Find the last point before `x` or the earliest point.
    /// E.g. for the curve containing [(0,0), (10,1)]:
    ///     find_last_point_before_x(-3) -> 0
    ///     find_last_point_before_x(3) -> 0
    ///     find_last_point_before_x(12) -> 10
    fn find_last_point_before_x(&self, x: X) -> Option<X> {
        let mut point_xs = self
            .points
            .keys()
            .into_iter()
            .map(Clone::clone)
            .collect::<Vec<_>>();
        point_xs.sort();
        point_xs
            .into_iter()
            .filter(|xi| *xi <= x) // 0 3
            .collect::<Vec<_>>()
            .into_iter()
            .last() // 3
            .or(self.points.keys().into_iter().min().map(Clone::clone))
            .map(|x| x.clone())
    }

    /// Find the first point after `x` or the latest point.
    /// E.g. for the curve containing [(0,0), (10,1)]:
    ///     find_first_point_after_x(-3) -> 0
    ///     find_first_point_after_x(3) -> 10
    ///     find_first_point_after_x(12) -> 10
    fn find_first_point_after_x(&self, x: X) -> Option<X> {
        let mut point_xs = self
            .points
            .keys()
            .into_iter()
            .map(Clone::clone)
            .collect::<Vec<_>>();
        point_xs.sort();
        point_xs
            .into_iter()
            .filter(|xi| x <= *xi)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .last()
            .or(self.points.keys().into_iter().max().map(Clone::clone))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl LinearInterp for f32 {
        fn norm_scale(self, x: f32) -> Self {
            self * x
        }
    }

    impl LinearInterp for i32 {
        fn norm_scale(self, x: f32) -> Self {
            ((self as f32) * x).floor() as i32
        }
    }

    #[test]
    fn test_find_last_point_before_x() {
        let points = HashMap::from([(0i16, 0f32), (3, 3f32), (10, 10f32)]);
        let curve = Curve::new(points).unwrap();

        assert_eq!(curve.find_last_point_before_x(-3), Some(0));
        assert_eq!(curve.find_last_point_before_x(0), Some(0));
        assert_eq!(curve.find_last_point_before_x(1), Some(0));
        assert_eq!(curve.find_last_point_before_x(3), Some(3));
        assert_eq!(curve.find_last_point_before_x(4), Some(3));
        assert_eq!(curve.find_last_point_before_x(10), Some(10));
        assert_eq!(curve.find_last_point_before_x(100), Some(10));
    }

    #[test]
    fn test_find_first_point_after_x() {
        let points = HashMap::from([(0i16, 0), (3, 3), (10, 10)]);
        let curve = Curve::new(points).unwrap();

        assert_eq!(curve.find_first_point_after_x(-3), Some(0));
        assert_eq!(curve.find_first_point_after_x(0), Some(0));
        assert_eq!(curve.find_first_point_after_x(1), Some(3));
        assert_eq!(curve.find_first_point_after_x(3), Some(3));
        assert_eq!(curve.find_first_point_after_x(4), Some(10));
        assert_eq!(curve.find_first_point_after_x(10), Some(10));
        assert_eq!(curve.find_first_point_after_x(100), Some(10));
    }

    #[test]
    fn test_lookup() {
        let points = HashMap::from([(0i16, 0f32), (3, 3f32), (10, 10f32)]);
        let curve = Curve::new(points).unwrap();

        assert_eq!(curve.lookup(-3i16), 0f32);
        assert_eq!(curve.lookup(0i16), 0f32);
        assert_eq!(curve.lookup(1i16), 1f32);
        assert_eq!(curve.lookup(3i16), 3f32);
        assert_eq!(curve.lookup(10i16), 10f32);
        assert_eq!(curve.lookup(100i16), 10f32);
    }
}
