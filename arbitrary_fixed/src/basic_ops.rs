use std::ops::Rem;
use std::ops::{Add, Div, Mul, Neg, Sub};

use cgmath::num_traits::Inv;

use crate::ArbitraryFixed;
use crate::SCALING_FACTOR;
use crate::SIZE;

impl Add for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn add(self, other: ArbitraryFixed) -> Self::Output {
        let mut ret: ArbitraryFixed = Default::default();

        let mut carry_prev = false;
        for ((r, a), b) in ret.data.iter_mut().zip(self.data).zip(other.data) {
            *r = a.wrapping_add(b);
            let carry = *r < a;
            *r = (carry_prev as u32).wrapping_add(*r);
            carry_prev = carry || (carry_prev && (*r == 0));
        }

        ret
    }
}

impl Neg for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn neg(self) -> Self::Output {
        let mut ret: ArbitraryFixed = Default::default();

        let mut carry_prev = true;
        for (r, a) in ret.data.iter_mut().zip(self.data) {
            *r = !a;
            *r = (carry_prev as u32).wrapping_add(*r);
            carry_prev = carry_prev && (*r == 0);
        }

        ret
    }
}

impl Sub for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn sub(self, rhs: ArbitraryFixed) -> Self::Output {
        self + -rhs
    }
}

impl Mul for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn mul(self, other: ArbitraryFixed) -> Self::Output {
        let a_negative = self.is_negative();
        let b_negative = other.is_negative();

        let fix_a = match a_negative {
            true => -self,
            false => self,
        };

        let fix_b = match b_negative {
            true => -other,
            false => other,
        };

        let mut res: [u32; SIZE * 2] = Default::default();

        for (i, a) in fix_a.data.iter().enumerate() {
            let mut carry = 0;
            for (j, b) in fix_b.data.iter().enumerate() {
                let product = (*a as u64) * (*b as u64) + (res[i + j] as u64) + (carry as u64);
                res[i + j] = product as u32;
                carry = (product >> 32) as u32
            }
            res[i + SIZE] = carry;
        }

        let mut ret: ArbitraryFixed = Default::default();

        for (idx, r) in ret.data.iter_mut().enumerate().rev() {
            *r = (if (SCALING_FACTOR % 32) > 0 {
                res[idx + 1 + SCALING_FACTOR / 32]
                    << ((32 as usize).wrapping_sub(SCALING_FACTOR) % 32)
            } else {
                0
            }) | ((res[idx + (SCALING_FACTOR / 32)]) >> (SCALING_FACTOR % 32));
        }

        match a_negative != b_negative {
            true => -ret,
            false => ret,
        }
    }
}

impl Div for ArbitraryFixed {
    type Output = ArbitraryFixed;

    // Goldschmidt's Algorithm!
    fn div(self, other: ArbitraryFixed) -> ArbitraryFixed {
        let a_negative = other.is_negative();
        let b_negative = self.is_negative();

        let mut n = match b_negative {
            true => -self,
            false => self,
        };

        let mut d = match a_negative {
            true => -other,
            false => other,
        };

        let msb = d.msb();

        // Dividing by zero is bad juju
        if msb == -1 {
            return d;
        }

        let offset = msb + 1 - SCALING_FACTOR as isize;

        n = match offset > 0 {
            true => n.rshift(offset as usize),
            false => n.lshift((-offset) as usize),
        };

        d = match offset > 0 {
            true => d.rshift(offset as usize),
            false => d.lshift((-offset) as usize),
        };

        let mut f = ArbitraryFixed::from(2.82352941176) - ArbitraryFixed::from(1.88235294118) * d;
        let two = ArbitraryFixed::from(2u32);
        const PRECISION: usize = 6;
        for _ in 0..PRECISION {
            n = f * n;
            d = f * d;
            f = two - d;
        }

        match a_negative != b_negative {
            true => -n,
            false => n,
        }
    }
}

impl Inv for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn inv(self) -> Self::Output {
        let a_negative = self.is_negative();

        let fix_a = match a_negative {
            true => -self,
            false => self,
        };

        let mut ret: ArbitraryFixed = Default::default();

        let msb = fix_a.msb();

        // Dividing by zero is bad juju
        if msb == -1 {
            return ret;
        }

        let guess = (-(msb + 1) + 2 * (SCALING_FACTOR as isize)) as usize;

        ret.data[(guess as usize) / 32] = 1 << ((guess) & 0x1F);

        const ITERATIONS: u32 = 8;
        let fix_two: ArbitraryFixed = 2u32.into();
        for _ in 0..ITERATIONS {
            ret = ret * (fix_two - fix_a * ret)
        }

        match a_negative {
            true => -ret,
            false => ret,
        }
    }
}

impl Rem for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn rem(self, rhs: ArbitraryFixed) -> Self::Output {
        let mut a = self / rhs;

        a.data[SCALING_FACTOR / 32] &= 0xFFFFFFFF << (SCALING_FACTOR % 32);
        for i in 0..(SCALING_FACTOR / 32) {
            a.data[i] = 0;
        }

        self - a * rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let (fa, fb): (f32, f32) = (3.0, 5.3);
        let (a, b): (ArbitraryFixed, ArbitraryFixed) = (fa.into(), fb.into());
        assert_eq!(a + b, (fa + fb).into());
    }

    #[test]
    fn test_sub() {
        let (fa, fb): (f32, f32) = (3.0, 5.3);
        let (a, b): (ArbitraryFixed, ArbitraryFixed) = (fa.into(), fb.into());
        assert_eq!(a - b, (fa - fb).into());
    }

    #[test]
    fn test_mul() {
        let (fa, fb): (f32, f32) = (3.0, 5.3);
        let (a, b): (ArbitraryFixed, ArbitraryFixed) = (fa.into(), fb.into());
        assert_eq!(a * b, (fa * fb).into());
    }

    #[test]
    fn test_div() {
        let fa: f32 = 1.0;
        let fb: f32 = 1481000.0;
        let a: ArbitraryFixed = fa.into();
        let b: ArbitraryFixed = fb.into();
        assert_eq!(f32::from(a / b), fa / fb);
    }

    #[test]
    fn test_inv() {
        let fa: f32 = 5.4;
        let a: ArbitraryFixed = fa.into();
        assert_eq!(f32::from(a.inv()), (1.0 / fa));
    }

    #[test]
    fn test_rem() {
        let fa: f32 = 3.0;
        let fb: f32 = 5.3;
        let a: ArbitraryFixed = fa.into();
        let b: ArbitraryFixed = fb.into();
        assert_eq!(f32::from(a % b), (fa % fb));
    }
}
