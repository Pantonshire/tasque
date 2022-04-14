use std::error;
use std::fmt;
use std::fmt::Write;
use std::num::NonZeroU8;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Component<const N: u8> {
    start: u8,
    end: u8,
    step: NonZeroU8,
}

impl<const N: u8> Component<N> {
    const NONZERO_1: NonZeroU8 = unsafe { NonZeroU8::new_unchecked(1) };

    pub(crate) fn new(start: u8, end: u8, step: NonZeroU8) -> Result<Self, Error> {
        if start <= end && end <= N {
            Ok(Self { start, end, step })
        } else {
            Err(Error)
        }
    }

    pub(crate) fn exactly_zero() -> Self {
        match Self::exactly(0) {
            Ok(c) => c,
            Err(_) => unreachable!(),
        }
    }

    pub(crate) fn exactly(val: u8) -> Result<Self, Error> {
        Self::new(val, val, Self::NONZERO_1)
    }

    pub(crate) fn between(start: u8, end: u8) -> Result<Self, Error> {
        Self::new(start, end, Self::NONZERO_1)
    }

    pub(crate) fn every() -> Self {
        Self::every_step(Self::NONZERO_1)
    }

    pub(crate) fn every_step(step: NonZeroU8) -> Self {
        match Self::new(0, N, step) {
            Ok(c) => c,
            Err(_) => unreachable!(),
        }
    }

    pub(crate) fn min_value(self) -> u8 {
        self.start
    }

    pub(crate) fn min_value_bounded(self, lower_bound: u8) -> Option<u8> {
        if lower_bound <= self.start {
            return Some(self.start);
        }

        let bound_offset = lower_bound - self.start;
        let num_steps = div_ceil(bound_offset, self.step);
        let min_value_offset = num_steps.checked_mul(self.step.get())?;
        let min_value = self.start.checked_add(min_value_offset)?;

        if min_value <= self.end {
            Some(min_value)
        } else {
            None
        }
    }
}

impl<const N: u8> Default for Component<N> {
    fn default() -> Self {
        Self::every()
    }
}

impl<const N: u8> fmt::Display for Component<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)?;
        } else if self.start == 0 && self.end == N {
            f.write_char('*')?;
        } else {
            write!(f, "{}-{}", self.start, self.end)?;
        }

        if self.step.get() != 1 {
            write!(f, "/{}", self.step.get())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid range for schedule component")
    }
}

impl error::Error for Error {}

fn div_ceil(lhs: u8, rhs: NonZeroU8) -> u8 {
    (lhs / rhs) + u8::from((lhs % rhs) != 0)
}

#[cfg(test)]
#[allow(clippy::pedantic)]
mod tests {
    use std::num::NonZeroU8;

    use super::Component;

    #[test]
    fn test_min_value() {
        assert_eq!(Component::<59>::every().min_value(), 0);
        assert_eq!(Component::<59>::new(5, 10, NonZeroU8::new(1).unwrap()).unwrap().min_value(), 5);
        assert_eq!(Component::<59>::new(5, 10, NonZeroU8::new(2).unwrap()).unwrap().min_value(), 5);
    }

    #[test]
    fn test_min_value_bounded() {
        assert_eq!(Component::<59>::every().min_value_bounded(0), Some(0));
        assert_eq!(Component::<59>::every().min_value_bounded(1), Some(1));
        assert_eq!(Component::<59>::every().min_value_bounded(59), Some(59));
        assert_eq!(Component::<59>::every().min_value_bounded(60), None);

        assert_eq!(Component::<59>::every_step(NonZeroU8::new(5).unwrap()).min_value_bounded(0), Some(0));
        assert_eq!(Component::<59>::every_step(NonZeroU8::new(5).unwrap()).min_value_bounded(1), Some(5));
        assert_eq!(Component::<59>::every_step(NonZeroU8::new(5).unwrap()).min_value_bounded(5), Some(5));
        assert_eq!(Component::<59>::every_step(NonZeroU8::new(5).unwrap()).min_value_bounded(6), Some(10));
        assert_eq!(Component::<59>::every_step(NonZeroU8::new(5).unwrap()).min_value_bounded(10), Some(10));
        assert_eq!(Component::<59>::every_step(NonZeroU8::new(5).unwrap()).min_value_bounded(11), Some(15));
        assert_eq!(Component::<59>::every_step(NonZeroU8::new(5).unwrap()).min_value_bounded(55), Some(55));
        assert_eq!(Component::<59>::every_step(NonZeroU8::new(5).unwrap()).min_value_bounded(56), None);

        assert_eq!(Component::<59>::between(5, 9).unwrap().min_value_bounded(0), Some(5));
        assert_eq!(Component::<59>::between(5, 9).unwrap().min_value_bounded(1), Some(5));
        assert_eq!(Component::<59>::between(5, 9).unwrap().min_value_bounded(5), Some(5));
        assert_eq!(Component::<59>::between(5, 9).unwrap().min_value_bounded(6), Some(6));
        assert_eq!(Component::<59>::between(5, 9).unwrap().min_value_bounded(9), Some(9));
        assert_eq!(Component::<59>::between(5, 9).unwrap().min_value_bounded(10), None);

        assert_eq!(Component::<59>::new(5, 9, NonZeroU8::new(2).unwrap()).unwrap().min_value_bounded(0), Some(5));
        assert_eq!(Component::<59>::new(5, 9, NonZeroU8::new(2).unwrap()).unwrap().min_value_bounded(1), Some(5));
        assert_eq!(Component::<59>::new(5, 9, NonZeroU8::new(2).unwrap()).unwrap().min_value_bounded(5), Some(5));
        assert_eq!(Component::<59>::new(5, 9, NonZeroU8::new(2).unwrap()).unwrap().min_value_bounded(6), Some(7));
        assert_eq!(Component::<59>::new(5, 9, NonZeroU8::new(2).unwrap()).unwrap().min_value_bounded(7), Some(7));
        assert_eq!(Component::<59>::new(5, 9, NonZeroU8::new(2).unwrap()).unwrap().min_value_bounded(8), Some(9));
        assert_eq!(Component::<59>::new(5, 9, NonZeroU8::new(2).unwrap()).unwrap().min_value_bounded(9), Some(9));
        assert_eq!(Component::<59>::new(5, 9, NonZeroU8::new(2).unwrap()).unwrap().min_value_bounded(10), None);
        
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(0), Some(30));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(1), Some(30));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(30), Some(30));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(31), Some(37));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(37), Some(37));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(38), Some(44));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(44), Some(44));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(45), Some(51));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(58), Some(58));
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(59), None);
        assert_eq!(Component::<59>::new(30, 59, NonZeroU8::new(7).unwrap()).unwrap().min_value_bounded(60), None);

        for i in 0..=18 {
            assert_eq!(Component::<59>::exactly(18).unwrap().min_value_bounded(i), Some(18));
        }
        for i in 19..=60 {
            assert_eq!(Component::<59>::exactly(18).unwrap().min_value_bounded(i), None);
        }
    }
}
