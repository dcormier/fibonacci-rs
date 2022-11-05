#![cfg(all(test, feature = "bench"))]
#![feature(test)]

extern crate test;

use core::fmt::Debug;

use fibs::Fibonacci;
use num::{BigUint, CheckedAdd, One, Zero};
use test::Bencher;

fn max<T>(b: &mut Bencher)
where
    T: Debug + Clone + CheckedAdd + Zero + One,
{
    let (max_n, _) = Fibonacci::<T>::f(usize::MAX).unwrap_err();

    b.iter(|| Fibonacci::<T>::f(max_n).unwrap())
}

macro_rules! bench_max {
    ($t:ty) => {
        paste::item! {
            #[bench]
            fn [<max_ $t>](b: &mut Bencher) {
                max::<$t>(b)
            }
        }
    };
}

bench_max!(i8);
bench_max!(i16);
bench_max!(i32);
bench_max!(i64);
bench_max!(i128);

bench_max!(u8);
bench_max!(u16);
bench_max!(u32);
bench_max!(u64);
bench_max!(u128);

#[bench]
fn bench_f_100_big_uint(b: &mut Bencher) {
    b.iter(|| Fibonacci::<BigUint>::f(100).unwrap())
}
