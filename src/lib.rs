use core::{fmt::Debug, iter::FusedIterator};

use num::{CheckedAdd, One, Zero};

/// An easy way to get numbers in the Fibonacci series.
///
/// # Examples
///
/// You can just get a number directly (F*ₙ*) using [`Fibonacci::f`]:
/// ```
/// use fibonacci::Fibonacci;
///
/// let n = Fibonacci::f(9);
/// assert_eq!(Ok(34), n);
/// ```
///
/// Or use [`Fibonacci`] as an [`Iterator`]:
/// ```
/// # use fibonacci::Fibonacci;
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
/// # use fibonacci::Fibonacci;
/// #
/// // The iterator starts with 0, so `.take(10).last()` == F₉
/// let n = Fibonacci::default().take(10).last();
/// assert_eq!(Some(34), n, "F₉ == 34");
/// assert_eq!(Fibonacci::f(9).ok(), n);
/// ```
///
/// If you know you need multiple values, it will be cheaper to reuse an [`Iterator`].
/// ```
/// # use fibonacci::Fibonacci;
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
/// # use fibonacci::Fibonacci;
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
/// # use fibonacci::Fibonacci;
/// #
/// let nums = Fibonacci::default().collect::<Vec<u16>>();
/// assert_eq!(
///     vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368],
///     nums,
/// );
/// ```
#[derive(Debug, Default, Copy, Clone)]
pub struct Fibonacci<T>(Option<T>, Option<T>)
where
    T: Debug + Copy + Clone + CheckedAdd + Zero + One;

impl<T> Fibonacci<T>
where
    T: Debug + Default + Copy + Clone + CheckedAdd + Zero + One,
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
    /// use fibonacci::Fibonacci;
    ///
    /// let n = Fibonacci::f(9);
    /// assert_eq!(Ok(34), n);
    /// ```
    ///
    /// If you try to get a number too big for your target numeric type, it'll tell you:
    /// ```
    /// # use fibonacci::Fibonacci;
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
    /// # use fibonacci::Fibonacci;
    /// #
    /// let n = Fibonacci::<u128>::f(187);
    /// assert_eq!(Err((186, 332825110087067562321196029789634457848)), n);
    ///
    /// let n = Fibonacci::<u128>::f(186);
    /// assert_eq!(Ok(332825110087067562321196029789634457848), n);
    /// ```
    ///
    /// If you don't care if you asked for a value too high for your target numeric type
    /// and just want a value:
    /// ```
    /// # use fibonacci::Fibonacci;
    /// #
    /// let n = Fibonacci::<u8>::f(9000).unwrap_or_else(|(_max_n, max_val)| max_val);
    /// assert_eq!(233, n);
    /// ```
    pub fn f(n: usize) -> Result<T, (usize, T)> {
        let m = Self::default()
            .take(n + 1)
            .enumerate()
            .last()
            .expect("How could a numeric type not even be able to get to 0?");

        match m.0 {
            i if i == n => Ok(m.1),
            _ => Err(m),
        }
    }
}

impl<T> Iterator for Fibonacci<T>
where
    T: Debug + Copy + Clone + CheckedAdd + Zero + One,
{
    type Item = T;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.1.is_none() && self.0.is_some() {
            // We're here because we overflowed on the previous iteration.
            // We're done.
            return None;
        }

        let current = self.1.or_else(|| Some(T::zero()));
        let next = current.and_then(|current| current.checked_add(&self.0.unwrap_or_else(T::one)));

        self.0 = current;
        core::mem::replace(&mut self.1, next).or_else(|| Some(T::zero()))
    }
}

impl<T> FusedIterator for Fibonacci<T> where T: Debug + Copy + Clone + CheckedAdd + Zero + One {}

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

    fn n_too_big<T>(max_n: usize, expect: T)
    where
        T: Debug + Default + Copy + Clone + PartialEq + CheckedAdd + Zero + One,
    {
        assert_eq!(Ok(expect), Fibonacci::f(max_n));
        assert_eq!(Err((max_n, expect)), Fibonacci::f(max_n + 1));
    }

    #[test]
    fn n_too_big_u8() {
        n_too_big(13, 233_u8);
    }

    #[test]
    fn n_too_big_u64() {
        n_too_big(93, 12200160415121876738_u64);
    }
}
