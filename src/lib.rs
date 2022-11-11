#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

//! Provides a function and an [`Iterator`] for getting Fibonacci numbers.
//!
//! See the docs on [`Fibonacci::f`] for a function to get a specific F*ₙ* value.
//!
//! See the docs on [`Fibonacci`] for an [`Iterator`].

use core::{fmt::Debug, iter::FusedIterator, mem};

use num::{CheckedAdd, One, Zero};

/// An easy way to get numbers in the Fibonacci series.
///
/// # Examples
///
/// You can just get a number directly (F*ₙ*) using [`Fibonacci::f`]:
/// ```
/// use fibs::Fibonacci;
///
/// let n = Fibonacci::f(9);
/// assert_eq!(Ok(34), n);
/// ```
///
/// Or use [`Fibonacci`] as an [`Iterator`]:
/// ```
/// # use fibs::Fibonacci;
/// #
/// let nums = Fibonacci::default()
///     .skip(3)
///     .take(5)
///     .collect::<Vec<_>>();
/// assert_eq!(
///     vec![2, 3, 5, 8, 13],
///     nums,
/// );
/// ```
///
/// ```
/// # use fibs::Fibonacci;
/// #
/// // The iterator starts with 0, so `.take(10).last()` == F₉
/// let n = Fibonacci::default().take(10).last();
/// assert_eq!(Some(34), n, "F₉ == 34");
/// assert_eq!(Fibonacci::f(9).ok(), n);
/// ```
///
/// If you know you need multiple values, it will be cheaper to reuse an [`Iterator`].
/// ```
/// # use fibs::Fibonacci;
/// #
/// let nums = Fibonacci::default()
///     .enumerate()
///     .filter_map(|(n, value)| match n {
///         3 | 4 | 9 => Some(value),
///         _ => None,
///     })
///     .take(3)
///     .collect::<Vec<u8>>();
/// assert_eq!(
///     vec![
///         Fibonacci::<u8>::f(3).unwrap(),
///         Fibonacci::f(4).unwrap(),
///         Fibonacci::f(9).unwrap(),
///     ],
///     nums,
/// );
/// assert_eq!(
///     vec![2, 3, 34],
///     nums,
/// );
/// ```
///
/// The [`Iterator`] will produce values until the target numeric type would
/// overflow:
/// ```
/// # use fibs::Fibonacci;
/// #
/// let nums: Vec<_> = Fibonacci::<u8>::default().collect();
/// assert_eq!(
///     vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233],
///     nums,
/// );
/// ```
///
/// Wider numeric types result in larger numbers:
/// ```
/// # use fibs::Fibonacci;
/// #
/// let nums = Fibonacci::default().collect::<Vec<u16>>();
/// assert_eq!(
///     vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368],
///     nums,
/// );
/// ```
pub struct Fibonacci<T> {
    previous: Option<T>,
    current: Option<T>,
}

impl<T> Debug for Fibonacci<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Fibonacci")
            .field("previous", &self.previous)
            .field("current", &self.current)
            .finish()
    }
}

impl<T> Copy for Fibonacci<T> where T: Copy {}

impl<T> Clone for Fibonacci<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            previous: self.previous.clone(),
            current: self.current.clone(),
        }
    }
}

// This does not require that `T` implement `Default`
impl<T> Default for Fibonacci<T> {
    fn default() -> Self {
        Self {
            previous: None,
            current: None,
        }
    }
}

impl<T> Fibonacci<T> {
    fn has_overflowed(&self) -> bool {
        // If there is no current number, but there is a previous one,
        // then the last iteration caused an overflow.
        self.current.is_none() && self.previous.is_some()
    }
}

impl<T> Fibonacci<T>
where
    T: Debug + Clone + CheckedAdd + Zero + One,
{
    /// Returns the F*ₙ* value in the Fibonacci series.
    ///
    /// If F*ₙ* would overflow `T`, then `Err` is returned with a tuple
    /// indicating the largest *ₙ* that would *not* overflow `T`, as
    /// well as what that F*ₙ* value is.
    ///
    /// If you know you need multiple values, it will be cheaper to use
    /// this type as an [`Iterator`].
    ///
    /// # Examples
    ///
    /// ```
    /// use fibs::Fibonacci;
    ///
    /// let n = Fibonacci::f(9);
    /// assert_eq!(Ok(34), n);
    /// ```
    ///
    /// If you try to get a number too big for your target numeric type, it'll tell you:
    /// ```
    /// # use fibs::Fibonacci;
    /// #
    /// let n = Fibonacci::<u8>::f(14);
    /// assert_eq!(Err((13, 233)), n);
    ///
    /// // But that `Err` value tell you what the maximum value for your type is.
    /// let n = Fibonacci::<u8>::f(13);
    /// assert_eq!(Ok(233), n);
    /// ```
    ///
    /// Wider target type, larger numbers:
    /// ```
    /// # use fibs::Fibonacci;
    /// #
    /// let n = Fibonacci::<u128>::f(187);
    /// assert_eq!(Err((186, 332825110087067562321196029789634457848)), n);
    ///
    /// let n = Fibonacci::<u128>::f(186);
    /// assert_eq!(Ok(332825110087067562321196029789634457848), n);
    ///
    #[cfg_attr(
        feature = "std",
        doc = r##"
// Or, if you want _really_ big numbers, you can use other types.
let n = Fibonacci::<num::BigUint>::f(1000);
assert_eq!(
    "Ok(43466557686937456435688527675040625802564660517371780402481729089536555417949051890403879840079255169295922593080322634775209689623239873322471161642996440906533187938298969649928516003704476137795166849228875)",
    format!("{:?}", n),
);
"##
    )]
    /// ```
    ///
    /// If you don't care if you asked for a value too high for your target numeric type
    /// and just want a value:
    /// ```
    /// # use fibs::Fibonacci;
    /// #
    /// let n: u8 = Fibonacci::f(9000).unwrap_or_else(|(_max_n, max_val)| max_val);
    /// assert_eq!(233, n);
    /// ```
    pub fn f(n: usize) -> Result<T, (usize, T)> {
        // There's a little bit of trickery here.
        //
        // We need to handle both `n == 0` and `n == usize::MAX`.
        //
        // With `n == 0`, we can't just `Self::default().enumerate().take(n).last()`
        // because it'll take 0 items and `.last()` will return `None`. So we _must_
        // call `.next()` at least once.
        //
        // We can't use `Self::default().enumerate().take(n+1).last()` because that would
        // overflow (panic) if `n == usize::MAX`.
        //
        // This is how we handle both extremes.

        #[inline]
        fn zero_failed<T>() -> T {
            panic!(
                "How could numeric type {} not even be able to get to 0?",
                core::any::type_name::<T>()
            )
        }

        let mut iter = Self::default();

        // TODO: Once stabilized, use `Iterator::advance_by()`
        //       https://github.com/rust-lang/rust/issues/77404
        //       https://doc.rust-lang.org/nightly/core/iter/trait.Iterator.html#method.advance_by
        let mut last = None;
        for i in 0..n {
            last = Some(iter.next().ok_or_else(|| {
                if i == 0 {
                    zero_failed()
                } else {
                    (
                        i - 1,
                        // If we're here, then this _must_ be `Some` because we've
                        // already done at least one iteration and didn't return.
                        last.unwrap(),
                    )
                }
            })?);
        }

        iter.next().ok_or_else(|| {
            if n == 0 {
                zero_failed()
            } else {
                (
                    n - 1,
                    // This _must_ be `Some` because n > 0 and we didn't return
                    // from the above `for` loop.
                    last.unwrap(),
                )
            }
        })
    }
}

impl<T> Iterator for Fibonacci<T>
where
    T: Clone + CheckedAdd + Zero + One,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_overflowed() {
            return None;
        }

        let current = self.current.clone().or_else(|| Some(T::zero()));
        let next = current
            .clone()
            .and_then(|current| current.checked_add(&self.previous.take().unwrap_or_else(T::one)));

        self.previous = current;
        mem::replace(&mut self.current, next).or_else(|| Some(T::zero()))
    }
}

impl<T> FusedIterator for Fibonacci<T> where T: Clone + CheckedAdd + Zero + One {}

#[cfg(test)]
mod test {
    use core::fmt::Debug;

    use num::{CheckedAdd, One, Zero};

    use super::Fibonacci;

    #[test]
    fn sanity() {
        assert_eq!(Ok(0), Fibonacci::f(0));
        assert_eq!(Ok(1), Fibonacci::f(1));
        assert_eq!(Ok(1), Fibonacci::f(2));
        assert_eq!(Ok(2), Fibonacci::f(3));

        assert_eq!(
            Ok(233),
            Fibonacci::<u8>::f(13),
            "Must be able to get the largest value that will fit in the target type"
        );

        let mut f = Fibonacci::default();
        assert_eq!(Some(0), f.next());
        assert_eq!(Some(1), f.next());
        assert_eq!(Some(1), f.next());
        assert_eq!(Some(2), f.next());
    }

    #[track_caller]
    fn known_max<T>(max_n: usize, expect: T)
    where
        T: Debug + Default + Clone + PartialEq + CheckedAdd + Zero + One,
    {
        assert_eq!(Ok(expect.clone()), Fibonacci::f(max_n));
        assert_eq!(Err((max_n, expect.clone())), Fibonacci::f(max_n + 1));
        assert_eq!(Err((max_n, expect)), Fibonacci::f(usize::MAX));
    }

    macro_rules! known_max {
        ($t:ty, $max_n:expr, $expect:expr) => {
            paste::item! {
                #[test]
                fn [<max_n_ $t>]() {
                    known_max::<$t>($max_n, $expect)
                }
            }
        };
    }

    known_max!(i8, 11, 89);
    known_max!(u8, 13, 233);
    known_max!(i16, 23, 28657);
    known_max!(u16, 24, 46368);
    known_max!(i32, 46, 1836311903);
    known_max!(u32, 47, 2971215073);
    known_max!(i64, 92, 7540113804746346429);
    known_max!(u64, 93, 12200160415121876738);
    known_max!(i128, 184, 127127879743834334146972278486287885163);
    known_max!(u128, 186, 332825110087067562321196029789634457848);

    #[cfg(feature = "std")]
    #[ignore = "Only run this if you've got time on your hands. \
        It takes at least the better part of an hour."]
    #[test]
    fn max_big_uint() {
        assert_eq!(
            Ok(()),
            Fibonacci::<num::BigUint>::f(usize::MAX)
                // Discarding the `BigUint` value because it's too much.
                .map(|_f| ())
                .map_err(|(max_n, _max_big_uint)| { max_n }),
            "Should be able to count to `usize::MAX`"
        );
    }
}
