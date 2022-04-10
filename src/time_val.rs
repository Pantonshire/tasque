use std::fmt;
use std::num::NonZeroU8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeVal<const N: u8> {
    At(SpecificTime<N>),
    Every,
}

impl<const N: u8> Default for TimeVal<N> {
    fn default() -> Self {
        Self::Every
    }
}

impl<const N: u8> TimeVal<N> {
    #[must_use]
    pub const fn at(n: u8) -> Self {
        Self::At(SpecificTime::new(n))
    }

    #[must_use]
    pub const fn every() -> Self {
        Self::Every
    }

    /// Returns the smallest unit of time which satisfies this `TimeVal`.
    #[must_use]
    pub const fn min_valid(self) -> u8 {
        match self {
            TimeVal::At(t) => t.get(),
            TimeVal::Every => 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SpecificTime<const N: u8>(NonZeroU8);

impl<const N: u8> SpecificTime<N> {
    /// # Panics
    /// Panics if the generic argument `N` is zero.
    #[inline]
    #[must_use]
    pub const fn new(n: u8) -> Self {
        // `SpecificTime<N>` represents a unit of time in the range [0,N), so if `N` is zero, then
        // there are no possible times that can be stored so this type is empty and useless.
        // Unfortunately, there is currently no way to assert that `N > 0` at compile time in
        // stable rust (1.60.0 at the time of writing), so we assert it here instead. This should
        // not incur any runtime cost because the compiler should detect that this is dead code
        // after monomorphising for all `N > 0` and remove it.
        assert!(N > 0);

        // SAFETY:
        // The only safety condition of `NonZeroU8::new_unchecked` is that its input is not zero.
        // A proof by contradiction that this condition holds:
        // - Assume that `!(n % N) == 0`.
        // - Negating both sides gives `!!(n % N) == 0xFF`.
        // - By involution of bitwise negation, `n % N == 0xFF`.
        // - `n % N < N` by definition of the remainder operator for integers.
        // - `N <= 0xFF` because it is a `u8`. Therefore, there are two possible cases: `N < 0xFF`
        //   or `N == 0xFF`.
        // - If `N < 0xFF`, `n % N < 0xFF` by transitivity of the < relation. If `N == 0xFF`, then
        //   `n % N < 0xFF` by substitution. Therefore, `n % N < 0xFF`.
        // - We have both `n % N < 0xFF` and `n % N == 0xFF`, which is a contradiction. Therefore,
        //   the initial assumption that `!(n % N) == 0` cannot be true.
        let repr = unsafe { NonZeroU8::new_unchecked(!(n % N)) };

        // We store the negation of `n % N` as a `NonZeroU8`. This allows us to "have our cake and
        // eat it"; we can get an efficient 1-byte representation for `TimeVal` because `NonZeroU8`
        // uses `#[rustc_layout_scalar_valid_range_start(1)]`, and we can also represent zero time
        // values because we are mapping zero to a non-zero value by doing the negation.
        Self(repr)
    }

    #[inline]
    #[must_use]
    pub const fn get(self) -> u8 {
        !self.0.get()
    }
}

impl<const N: u8> fmt::Debug for SpecificTime<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

#[cfg(test)]
#[allow(clippy::pedantic)]
mod tests {
    use super::{SpecificTime, TimeVal};

    #[test]
    pub fn test_specific_time_get() {
        for n in 0..=u8::MAX {
            assert_eq!(SpecificTime::<60>::new(n).get(), n % 60);
            assert_eq!(SpecificTime::<24>::new(n).get(), n % 24);
            assert_eq!(SpecificTime::<31>::new(n).get(), n % 31);
        }
    }

    #[test]
    pub fn test_time_val_min_valid() {
        assert_eq!(TimeVal::<60>::Every.min_valid(), 0);
        for n in 0..u8::MAX {
            assert_eq!(TimeVal::<60>::At(SpecificTime::new(n)).min_valid(), n % 60);
        }

        assert_eq!(TimeVal::<24>::Every.min_valid(), 0);
        for n in 0..u8::MAX {
            assert_eq!(TimeVal::<24>::At(SpecificTime::new(n)).min_valid(), n % 24);
        }

        assert_eq!(TimeVal::<31>::Every.min_valid(), 0);
        for n in 0..u8::MAX {
            assert_eq!(TimeVal::<31>::At(SpecificTime::new(n)).min_valid(), n % 31);
        }
    }

    #[test]
    pub fn test_time_val_small() {
        use std::mem::size_of;
        assert_eq!(size_of::<TimeVal::<60>>(), 1);
        assert_eq!(size_of::<TimeVal::<24>>(), 1);
        assert_eq!(size_of::<TimeVal::<31>>(), 1);
    }
}
