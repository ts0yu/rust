// Copyright 2012-2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![stable(feature = "duration_core", since = "1.25.0")]

//! Temporal quantification.
//!
//! Example:
//!
//! ```
//! use std::time::Duration;
//!
//! let five_seconds = Duration::new(5, 0);
//! // both declarations are equivalent
//! assert_eq!(Duration::new(5, 0), Duration::from_secs(5));
//! ```

use {fmt, u64};
use iter::Sum;
use ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign};

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const NANOS_PER_MICRO: u32 = 1_000;
const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;
const MAX_NANOS_F64: f64 = ((u64::MAX as u128)*(NANOS_PER_SEC as u128)) as f64;

/// A `Duration` type to represent a span of time, typically used for system
/// timeouts.
///
/// Each `Duration` is composed of a whole number of seconds and a fractional part
/// represented in nanoseconds.  If the underlying system does not support
/// nanosecond-level precision, APIs binding a system timeout will typically round up
/// the number of nanoseconds.
///
/// `Duration`s implement many common traits, including [`Add`], [`Sub`], and other
/// [`ops`] traits.
///
/// [`Add`]: ../../std/ops/trait.Add.html
/// [`Sub`]: ../../std/ops/trait.Sub.html
/// [`ops`]: ../../std/ops/index.html
///
/// # Examples
///
/// ```
/// use std::time::Duration;
///
/// let five_seconds = Duration::new(5, 0);
/// let five_seconds_and_five_nanos = five_seconds + Duration::new(0, 5);
///
/// assert_eq!(five_seconds_and_five_nanos.as_secs(), 5);
/// assert_eq!(five_seconds_and_five_nanos.subsec_nanos(), 5);
///
/// let ten_millis = Duration::from_millis(10);
/// ```
#[stable(feature = "duration", since = "1.3.0")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration {
    secs: u64,
    nanos: u32, // Always 0 <= nanos < NANOS_PER_SEC
}

impl Duration {
    /// Creates a new `Duration` from the specified number of whole seconds and
    /// additional nanoseconds.
    ///
    /// If the number of nanoseconds is greater than 1 billion (the number of
    /// nanoseconds in a second), then it will carry over into the seconds provided.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the carry from the nanoseconds overflows
    /// the seconds counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let five_seconds = Duration::new(5, 0);
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    pub fn new(secs: u64, nanos: u32) -> Duration {
        let secs = secs.checked_add((nanos / NANOS_PER_SEC) as u64)
            .expect("overflow in Duration::new");
        let nanos = nanos % NANOS_PER_SEC;
        Duration { secs: secs, nanos: nanos }
    }

    /// Creates a new `Duration` from the specified number of whole seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_secs(5);
    ///
    /// assert_eq!(5, duration.as_secs());
    /// assert_eq!(0, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    pub const fn from_secs(secs: u64) -> Duration {
        Duration { secs: secs, nanos: 0 }
    }

    /// Creates a new `Duration` from the specified number of milliseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(2569);
    ///
    /// assert_eq!(2, duration.as_secs());
    /// assert_eq!(569_000_000, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    pub const fn from_millis(millis: u64) -> Duration {
        Duration {
            secs: millis / MILLIS_PER_SEC,
            nanos: ((millis % MILLIS_PER_SEC) as u32) * NANOS_PER_MILLI,
        }
    }

    /// Creates a new `Duration` from the specified number of microseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_micros(1_000_002);
    ///
    /// assert_eq!(1, duration.as_secs());
    /// assert_eq!(2000, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration_from_micros", since = "1.27.0")]
    #[inline]
    pub const fn from_micros(micros: u64) -> Duration {
        Duration {
            secs: micros / MICROS_PER_SEC,
            nanos: ((micros % MICROS_PER_SEC) as u32) * NANOS_PER_MICRO,
        }
    }

    /// Creates a new `Duration` from the specified number of nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_nanos(1_000_000_123);
    ///
    /// assert_eq!(1, duration.as_secs());
    /// assert_eq!(123, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration_extras", since = "1.27.0")]
    #[inline]
    pub const fn from_nanos(nanos: u64) -> Duration {
        Duration {
            secs: nanos / (NANOS_PER_SEC as u64),
            nanos: (nanos % (NANOS_PER_SEC as u64)) as u32,
        }
    }

    /// Returns the number of _whole_ seconds contained by this `Duration`.
    ///
    /// The returned value does not include the fractional (nanosecond) part of the
    /// duration, which can be obtained using [`subsec_nanos`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_secs(), 5);
    /// ```
    ///
    /// To determine the total number of seconds represented by the `Duration`,
    /// use `as_secs` in combination with [`subsec_nanos`]:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    ///
    /// assert_eq!(5.730023852,
    ///            duration.as_secs() as f64
    ///            + duration.subsec_nanos() as f64 * 1e-9);
    /// ```
    ///
    /// [`subsec_nanos`]: #method.subsec_nanos
    #[stable(feature = "duration", since = "1.3.0")]
    #[rustc_const_unstable(feature="duration_getters")]
    #[inline]
    pub const fn as_secs(&self) -> u64 { self.secs }

    /// Returns the fractional part of this `Duration`, in whole milliseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by milliseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one thousand).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(5432);
    /// assert_eq!(duration.as_secs(), 5);
    /// assert_eq!(duration.subsec_millis(), 432);
    /// ```
    #[stable(feature = "duration_extras", since = "1.27.0")]
    #[rustc_const_unstable(feature="duration_getters")]
    #[inline]
    pub const fn subsec_millis(&self) -> u32 { self.nanos / NANOS_PER_MILLI }

    /// Returns the fractional part of this `Duration`, in whole microseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by microseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one million).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_micros(1_234_567);
    /// assert_eq!(duration.as_secs(), 1);
    /// assert_eq!(duration.subsec_micros(), 234_567);
    /// ```
    #[stable(feature = "duration_extras", since = "1.27.0")]
    #[rustc_const_unstable(feature="duration_getters")]
    #[inline]
    pub const fn subsec_micros(&self) -> u32 { self.nanos / NANOS_PER_MICRO }

    /// Returns the fractional part of this `Duration`, in nanoseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by nanoseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one billion).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(5010);
    /// assert_eq!(duration.as_secs(), 5);
    /// assert_eq!(duration.subsec_nanos(), 10_000_000);
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[rustc_const_unstable(feature="duration_getters")]
    #[inline]
    pub const fn subsec_nanos(&self) -> u32 { self.nanos }

    /// Returns the total number of whole milliseconds contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(duration_as_u128)]
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_millis(), 5730);
    /// ```
    #[unstable(feature = "duration_as_u128", issue = "50202")]
    #[inline]
    pub fn as_millis(&self) -> u128 {
        self.secs as u128 * MILLIS_PER_SEC as u128 + (self.nanos / NANOS_PER_MILLI) as u128
    }

    /// Returns the total number of whole microseconds contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(duration_as_u128)]
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_micros(), 5730023);
    /// ```
    #[unstable(feature = "duration_as_u128", issue = "50202")]
    #[inline]
    pub fn as_micros(&self) -> u128 {
        self.secs as u128 * MICROS_PER_SEC as u128 + (self.nanos / NANOS_PER_MICRO) as u128
    }

    /// Returns the total number of nanoseconds contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(duration_as_u128)]
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_nanos(), 5730023852);
    /// ```
    #[unstable(feature = "duration_as_u128", issue = "50202")]
    #[inline]
    pub fn as_nanos(&self) -> u128 {
        self.secs as u128 * NANOS_PER_SEC as u128 + self.nanos as u128
    }

    /// Checked `Duration` addition. Computes `self + other`, returning [`None`]
    /// if overflow occurred.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 0).checked_add(Duration::new(0, 1)), Some(Duration::new(0, 1)));
    /// assert_eq!(Duration::new(1, 0).checked_add(Duration::new(std::u64::MAX, 0)), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[inline]
    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_add(rhs.secs) {
            let mut nanos = self.nanos + rhs.nanos;
            if nanos >= NANOS_PER_SEC {
                nanos -= NANOS_PER_SEC;
                if let Some(new_secs) = secs.checked_add(1) {
                    secs = new_secs;
                } else {
                    return None;
                }
            }
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration {
                secs,
                nanos,
            })
        } else {
            None
        }
    }

    /// Checked `Duration` subtraction. Computes `self - other`, returning [`None`]
    /// if the result would be negative or if overflow occurred.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 1).checked_sub(Duration::new(0, 0)), Some(Duration::new(0, 1)));
    /// assert_eq!(Duration::new(0, 0).checked_sub(Duration::new(0, 1)), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[inline]
    pub fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_sub(rhs.secs) {
            let nanos = if self.nanos >= rhs.nanos {
                self.nanos - rhs.nanos
            } else {
                if let Some(sub_secs) = secs.checked_sub(1) {
                    secs = sub_secs;
                    self.nanos + NANOS_PER_SEC - rhs.nanos
                } else {
                    return None;
                }
            };
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs: secs, nanos: nanos })
        } else {
            None
        }
    }

    /// Checked `Duration` multiplication. Computes `self * other`, returning
    /// [`None`] if overflow occurred.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 500_000_001).checked_mul(2), Some(Duration::new(1, 2)));
    /// assert_eq!(Duration::new(std::u64::MAX - 1, 0).checked_mul(2), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[inline]
    pub fn checked_mul(self, rhs: u32) -> Option<Duration> {
        // Multiply nanoseconds as u64, because it cannot overflow that way.
        let total_nanos = self.nanos as u64 * rhs as u64;
        let extra_secs = total_nanos / (NANOS_PER_SEC as u64);
        let nanos = (total_nanos % (NANOS_PER_SEC as u64)) as u32;
        if let Some(secs) = self.secs
            .checked_mul(rhs as u64)
            .and_then(|s| s.checked_add(extra_secs)) {
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration {
                secs,
                nanos,
            })
        } else {
            None
        }
    }

    /// Checked `Duration` division. Computes `self / other`, returning [`None`]
    /// if `other == 0`.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(2, 0).checked_div(2), Some(Duration::new(1, 0)));
    /// assert_eq!(Duration::new(1, 0).checked_div(2), Some(Duration::new(0, 500_000_000)));
    /// assert_eq!(Duration::new(2, 0).checked_div(0), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[inline]
    pub fn checked_div(self, rhs: u32) -> Option<Duration> {
        if rhs != 0 {
            let secs = self.secs / (rhs as u64);
            let carry = self.secs - secs * (rhs as u64);
            let extra_nanos = carry * (NANOS_PER_SEC as u64) / (rhs as u64);
            let nanos = self.nanos / rhs + (extra_nanos as u32);
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs: secs, nanos: nanos })
        } else {
            None
        }
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(rhs).expect("overflow when adding durations")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(rhs).expect("overflow when subtracting durations")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        self.checked_mul(rhs).expect("overflow when multiplying duration by scalar")
    }
}

#[stable(feature = "duration_mul_div_extras", since = "1.29.0")]
impl Mul<Duration> for u32 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Duration {
        rhs.checked_mul(self).expect("overflow when multiplying scalar by duration")
    }
}

#[stable(feature = "duration_mul_div_extras", since = "1.29.0")]
impl Mul<f64> for Duration {
    type Output = Duration;

    fn mul(self, rhs: f64) -> Duration {
        const NPS: f64 = NANOS_PER_SEC as f64;
        let nanos_f64 = rhs * (NPS * (self.secs as f64) + (self.nanos as f64));
        if !nanos_f64.is_finite() {
            panic!("got non-finite value when multiplying duration by float");
        }
        if nanos_f64 > MAX_NANOS_F64 {
            panic!("overflow when multiplying duration by float");
        }
        if nanos_f64 < 0.0 {
            panic!("underflow when multiplying duration by float");
        }
        let nanos_u128 = nanos_f64 as u128;
        Duration {
            secs: (nanos_u128 / (NANOS_PER_SEC as u128)) as u64,
            nanos: (nanos_u128 % (NANOS_PER_SEC as u128)) as u32,
        }
    }
}

#[stable(feature = "duration_mul_div_extras", since = "1.29.0")]
impl Mul<Duration> for f64 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Duration {
        const NPS: f64 = NANOS_PER_SEC as f64;
        let nanos_f64 = self * (NPS * (rhs.secs as f64) + (rhs.nanos as f64));
        if !nanos_f64.is_finite() {
            panic!("got non-finite value when multiplying float by duration");
        }
        if nanos_f64 > MAX_NANOS_F64 {
            panic!("overflow when multiplying float by duration");
        }
        if nanos_f64 < 0.0 {
            panic!("underflow when multiplying float by duration");
        }
        let nanos_u128 = nanos_f64 as u128;
        Duration {
            secs: (nanos_u128 / (NANOS_PER_SEC as u128)) as u64,
            nanos: (nanos_u128 % (NANOS_PER_SEC as u128)) as u32,
        }
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

#[stable(feature = "duration_mul_div_extras", since = "1.29.0")]
impl MulAssign<f64> for Duration {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        self.checked_div(rhs).expect("divide by zero error when dividing duration by scalar")
    }
}

#[stable(feature = "duration_mul_div_extras", since = "1.29.0")]
impl Div<f64> for Duration {
    type Output = Duration;

    fn div(self, rhs: f64) -> Duration {
        const NPS: f64 = NANOS_PER_SEC as f64;
        let nanos_f64 = (NPS * (self.secs as f64) + (self.nanos as f64)) / rhs;
        if !nanos_f64.is_finite() {
            panic!("got non-finite value when dividing duration by float");
        }
        if nanos_f64 > MAX_NANOS_F64 {
            panic!("overflow when dividing duration by float");
        }
        if nanos_f64 < 0.0 {
            panic!("underflow when multiplying duration by float");
        }
        let nanos_u128 = nanos_f64 as u128;
        Duration {
            secs: (nanos_u128 / (NANOS_PER_SEC as u128)) as u64,
            nanos: (nanos_u128 % (NANOS_PER_SEC as u128)) as u32,
        }
    }
}

#[stable(feature = "duration_mul_div_extras", since = "1.29.0")]
impl Div<Duration> for Duration {
    type Output = f64;

    fn div(self, rhs: Duration) -> f64 {
        const NPS: f64 = NANOS_PER_SEC as f64;
        let nanos1 = NPS * (self.secs as f64) + (self.nanos as f64);
        let nanos2 = NPS * (rhs.secs as f64) + (rhs.nanos as f64);
        nanos1/nanos2
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

#[stable(feature = "duration_mul_div_extras", since = "1.29.0")]
impl DivAssign<f64> for Duration {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}


macro_rules! sum_durations {
    ($iter:expr) => {{
        let mut total_secs: u64 = 0;
        let mut total_nanos: u64 = 0;

        for entry in $iter {
            total_secs = total_secs
                .checked_add(entry.secs)
                .expect("overflow in iter::sum over durations");
            total_nanos = match total_nanos.checked_add(entry.nanos as u64) {
                Some(n) => n,
                None => {
                    total_secs = total_secs
                        .checked_add(total_nanos / NANOS_PER_SEC as u64)
                        .expect("overflow in iter::sum over durations");
                    (total_nanos % NANOS_PER_SEC as u64) + entry.nanos as u64
                }
            };
        }
        total_secs = total_secs
            .checked_add(total_nanos / NANOS_PER_SEC as u64)
            .expect("overflow in iter::sum over durations");
        total_nanos = total_nanos % NANOS_PER_SEC as u64;
        Duration {
            secs: total_secs,
            nanos: total_nanos as u32,
        }
    }};
}

#[stable(feature = "duration_sum", since = "1.16.0")]
impl Sum for Duration {
    fn sum<I: Iterator<Item=Duration>>(iter: I) -> Duration {
        sum_durations!(iter)
    }
}

#[stable(feature = "duration_sum", since = "1.16.0")]
impl<'a> Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item=&'a Duration>>(iter: I) -> Duration {
        sum_durations!(iter)
    }
}

#[stable(feature = "duration_debug_impl", since = "1.27.0")]
impl fmt::Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /// Formats a floating point number in decimal notation.
        ///
        /// The number is given as the `integer_part` and a fractional part.
        /// The value of the fractional part is `fractional_part / divisor`. So
        /// `integer_part` = 3, `fractional_part` = 12 and `divisor` = 100
        /// represents the number `3.012`. Trailing zeros are omitted.
        ///
        /// `divisor` must not be above 100_000_000. It also should be a power
        /// of 10, everything else doesn't make sense. `fractional_part` has
        /// to be less than `10 * divisor`!
        fn fmt_decimal(
            f: &mut fmt::Formatter,
            mut integer_part: u64,
            mut fractional_part: u32,
            mut divisor: u32,
        ) -> fmt::Result {
            // Encode the fractional part into a temporary buffer. The buffer
            // only need to hold 9 elements, because `fractional_part` has to
            // be smaller than 10^9. The buffer is prefilled with '0' digits
            // to simplify the code below.
            let mut buf = [b'0'; 9];

            // The next digit is written at this position
            let mut pos = 0;

            // We keep writing digits into the buffer while there are non-zero
            // digits left and we haven't written enough digits yet.
            while fractional_part > 0 && pos < f.precision().unwrap_or(9) {
                // Write new digit into the buffer
                buf[pos] = b'0' + (fractional_part / divisor) as u8;

                fractional_part %= divisor;
                divisor /= 10;
                pos += 1;
            }

            // If a precision < 9 was specified, there may be some non-zero
            // digits left that weren't written into the buffer. In that case we
            // need to perform rounding to match the semantics of printing
            // normal floating point numbers. However, we only need to do work
            // when rounding up. This happens if the first digit of the
            // remaining ones is >= 5.
            if fractional_part > 0 && fractional_part >= divisor * 5 {
                // Round up the number contained in the buffer. We go through
                // the buffer backwards and keep track of the carry.
                let mut rev_pos = pos;
                let mut carry = true;
                while carry && rev_pos > 0 {
                    rev_pos -= 1;

                    // If the digit in the buffer is not '9', we just need to
                    // increment it and can stop then (since we don't have a
                    // carry anymore). Otherwise, we set it to '0' (overflow)
                    // and continue.
                    if buf[rev_pos] < b'9' {
                        buf[rev_pos] += 1;
                        carry = false;
                    } else {
                        buf[rev_pos] = b'0';
                    }
                }

                // If we still have the carry bit set, that means that we set
                // the whole buffer to '0's and need to increment the integer
                // part.
                if carry {
                    integer_part += 1;
                }
            }

            // Determine the end of the buffer: if precision is set, we just
            // use as many digits from the buffer (capped to 9). If it isn't
            // set, we only use all digits up to the last non-zero one.
            let end = f.precision().map(|p| ::cmp::min(p, 9)).unwrap_or(pos);

            // If we haven't emitted a single fractional digit and the precision
            // wasn't set to a non-zero value, we don't print the decimal point.
            if end == 0 {
                write!(f, "{}", integer_part)
            } else {
                // We are only writing ASCII digits into the buffer and it was
                // initialized with '0's, so it contains valid UTF8.
                let s = unsafe {
                    ::str::from_utf8_unchecked(&buf[..end])
                };

                // If the user request a precision > 9, we pad '0's at the end.
                let w = f.precision().unwrap_or(pos);
                write!(f, "{}.{:0<width$}", integer_part, s, width = w)
            }
        }

        // Print leading '+' sign if requested
        if f.sign_plus() {
            write!(f, "+")?;
        }

        if self.secs > 0 {
            fmt_decimal(f, self.secs, self.nanos, 100_000_000)?;
            f.write_str("s")
        } else if self.nanos >= 1_000_000 {
            fmt_decimal(f, self.nanos as u64 / 1_000_000, self.nanos % 1_000_000, 100_000)?;
            f.write_str("ms")
        } else if self.nanos >= 1_000 {
            fmt_decimal(f, self.nanos as u64 / 1_000, self.nanos % 1_000, 100)?;
            f.write_str("µs")
        } else {
            fmt_decimal(f, self.nanos as u64, 0, 1)?;
            f.write_str("ns")
        }
    }
}
