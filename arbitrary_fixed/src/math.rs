use crate::{ArbitraryFixed, SCALING_FACTOR};

impl ArbitraryFixed {
    pub fn sqrt(&self) -> Self {
        let mut ret: ArbitraryFixed = Default::default();

        let msb = self.msb();

        if msb == -1 {
            return ret;
        }

        let guess = (((msb - (SCALING_FACTOR as isize)) / 2) + SCALING_FACTOR as isize) as usize;

        ret.data[(guess as usize) / 32] = 1 << ((guess) & 0x1F);

        const ITERATIONS: u32 = 8;
        for _ in 0..ITERATIONS {
            ret = (ret * ret + *self) / (ret.lshift1())
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        let fa: f32 = 13318.0;
        let a: ArbitraryFixed = fa.into();
        assert_eq!(f32::from(a.sqrt()), fa.sqrt());
    }
}
