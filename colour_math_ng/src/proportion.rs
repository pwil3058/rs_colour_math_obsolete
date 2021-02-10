// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//use std::cmp::Ordering;
#[cfg(test)]
mod proportion_tests;

use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use num_traits::FromPrimitive;
use num_traits_plus::float_plus::*;

pub trait ProportionValidation: Sized + Copy {
    fn is_vp(self) -> bool;

    fn validated_p(self) -> Self {
        debug_assert!(self.is_vp());
        self
    }
}

pub trait SumValidation: Sized + Copy {
    fn is_vs(self) -> bool;

    fn validated_s(self) -> Self {
        debug_assert!(self.is_vs());
        self
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord,
)]
pub struct UFDFraction(pub(crate) u64);

impl UFDFraction {
    const DENOM: u64 = u32::MAX as u64;
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(Self::DENOM);
    pub const TWO: Self = Self(Self::DENOM * 2);
    pub const THREE: Self = Self(Self::DENOM * 3);

    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
        let me = f64::from(*self);
        let other = f64::from(*other);
        me.approx_eq(&other, max_diff)
    }
}

impl ProportionValidation for UFDFraction {
    fn is_vp(self) -> bool {
        self <= Self::ONE
    }
}

impl SumValidation for UFDFraction {
    fn is_vs(self) -> bool {
        self <= Self::THREE
    }
}

macro_rules! impl_ufdr_add_sub {
    ($op_name:ident, $op_fn:ident) => {
        impl $op_name for UFDFraction {
            type Output = Self;

            fn $op_fn(self, rhs: Self) -> Self {
                Self(self.0.$op_fn(rhs.0))
            }
        }
    };
}

impl_ufdr_add_sub!(Add, add);
impl_ufdr_add_sub!(Sub, sub);

impl Div for UFDFraction {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let mut ws: u128 = self.0 as u128 * Self::DENOM as u128;
        ws /= rhs.0 as u128;
        Self(ws as u64)
    }
}

impl Mul for UFDFraction {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut ws: u128 = self.0 as u128 * rhs.0 as u128;
        ws /= Self::DENOM as u128;
        Self(ws as u64)
    }
}

impl From<f64> for UFDFraction {
    fn from(arg: f64) -> Self {
        let one = f64::from_u64(Self::DENOM).unwrap();
        let val = u64::from_f64(arg * one).unwrap();
        Self(val)
    }
}

impl From<UFDFraction> for f64 {
    fn from(arg: UFDFraction) -> Self {
        let one = f64::from_u64(UFDFraction::DENOM).unwrap();
        f64::from_u64(arg.0).unwrap() / one
    }
}

impl From<f32> for UFDFraction {
    fn from(arg: f32) -> Self {
        let one = f32::from_u64(Self::DENOM).unwrap();
        let val = u64::from_f32(arg * one).unwrap();
        Self(val)
    }
}

impl From<UFDFraction> for f32 {
    fn from(arg: UFDFraction) -> Self {
        let one = f32::from_u64(UFDFraction::DENOM).unwrap();
        f32::from_u64(arg.0).unwrap() / one
    }
}

macro_rules! impl_unsigned_to_from {
    ($unsigned:ty) => {
        impl From<$unsigned> for UFDFraction {
            fn from(arg: $unsigned) -> Self {
                let val = arg as u64 * Self::DENOM / <$unsigned>::MAX as u64;
                Self(val)
            }
        }

        impl From<UFDFraction> for $unsigned {
            fn from(arg: UFDFraction) -> Self {
                debug_assert!(arg <= UFDFraction::ONE);
                let val = arg.0 * <$unsigned>::MAX as u64 / UFDFraction::DENOM;
                val as $unsigned
            }
        }
    };
}

impl_unsigned_to_from!(u8);
impl_unsigned_to_from!(u16);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chroma {
    Shade(UFDFraction),
    Tint(UFDFraction),
}

impl Chroma {
    pub const ZERO: Self = Self::Shade(UFDFraction::ZERO);
    pub const ONE: Self = Self::Tint(UFDFraction::ONE);

    pub fn is_zero(&self) -> bool {
        match self {
            Chroma::Shade(proportion) => *proportion == UFDFraction::ZERO,
            Chroma::Tint(proportion) => *proportion == UFDFraction::ZERO,
        }
    }

    pub fn proportion(&self) -> UFDFraction {
        match self {
            Chroma::Shade(proportion) => *proportion,
            Chroma::Tint(proportion) => *proportion,
        }
    }

    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
        match self {
            Chroma::Shade(proportion) => match other {
                Chroma::Shade(other_proportion) => proportion.approx_eq(other_proportion, max_diff),
                Chroma::Tint(_) => false,
            },
            Chroma::Tint(proportion) => match other {
                Chroma::Shade(_) => false,
                Chroma::Tint(other_proportion) => proportion.approx_eq(other_proportion, max_diff),
            },
        }
    }
}

impl ProportionValidation for Chroma {
    fn is_vp(self) -> bool {
        match self {
            Chroma::Shade(proportion) => proportion.is_vp(),
            Chroma::Tint(proportion) => proportion.is_vp(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prop(pub(crate) u64);

impl Prop {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX);

    #[cfg(test)]
    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
        let me = f64::from(*self);
        let other = f64::from(*other);
        me.approx_eq(&other, max_diff)
    }
}

impl From<f32> for Prop {
    fn from(arg: f32) -> Self {
        debug_assert!(arg <= 1.0);
        let one = f32::from_u64(u64::MAX).unwrap();
        let val = u64::from_f32(arg * one).unwrap();
        Self(val)
    }
}

impl From<Prop> for f32 {
    fn from(arg: Prop) -> Self {
        let one = f32::from_u64(u64::MAX).unwrap();
        f32::from_u64(arg.0).unwrap() / one
    }
}

impl From<f64> for Prop {
    fn from(arg: f64) -> Self {
        debug_assert!(arg <= 1.0);
        let one = f64::from_u64(u64::MAX).unwrap();
        let val = u64::from_f64(arg * one).unwrap();
        Self(val)
    }
}

impl From<Prop> for f64 {
    fn from(arg: Prop) -> Self {
        let one = f64::from_u64(u64::MAX).unwrap();
        f64::from_u64(arg.0).unwrap() / one
    }
}

macro_rules! impl_unsigned_to_from_prop {
    ($unsigned:ty) => {
        impl From<$unsigned> for Prop {
            fn from(arg: $unsigned) -> Self {
                let val = arg as u128 * u64::MAX as u128 / <$unsigned>::MAX as u128;
                Self(val as u64)
            }
        }

        impl From<Prop> for $unsigned {
            fn from(arg: Prop) -> Self {
                let val = arg.0 as u128 * <$unsigned>::MAX as u128 / u64::MAX as u128;
                val as $unsigned
            }
        }
    };
}

impl_unsigned_to_from_prop!(u8);
impl_unsigned_to_from_prop!(u16);
impl_unsigned_to_from_prop!(u32);

impl Mul for Prop {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(((self.0 as u128 * rhs.0 as u128) / u64::MAX as u128) as u64)
    }
}

impl Div for Prop {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        debug_assert!(self.0 <= rhs.0);
        let result = (self.0 as u128 * u64::MAX as u128) / rhs.0 as u128;
        Self(result as u64)
    }
}

impl Add for Prop {
    type Output = Sum;

    fn add(self, rhs: Self) -> Sum {
        Sum(self.0 as u128 + rhs.0 as u128)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sum(pub(crate) u128);

impl Sum {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX as u128);
    pub const TWO: Self = Self(u64::MAX as u128 * 2);
    pub const THREE: Self = Self(u64::MAX as u128 * 3);

    pub fn is_valid(self) -> bool {
        self <= Self::THREE
    }

    #[cfg(test)]
    pub fn approx_eq(&self, other: &Self, max_diff: Option<f64>) -> bool {
        let me = f64::from(*self);
        let other = f64::from(*other);
        me.approx_eq(&other, max_diff)
    }
}

// impl From<f32> for Sum {
//     fn from(arg: f32) -> Self {
//         debug_assert!(arg <= 3.0);
//         let one = f32::from_u128(u64::MAX as u128).unwrap();
//         let val = u128::from_f32(arg * one).unwrap();
//         Self(val)
//     }
// }
//
// impl From<Sum> for f32 {
//     fn from(arg: Sum) -> Self {
//         let one = f32::from_u128(u64::MAX as u128).unwrap();
//         f32::from_u128(arg.0).unwrap() / one
//     }
// }

impl From<f64> for Sum {
    fn from(arg: f64) -> Self {
        debug_assert!(arg <= 3.0);
        let one = f64::from_u128(u64::MAX as u128).unwrap();
        let val = u128::from_f64(arg * one).unwrap();
        Self(val)
    }
}

impl From<Sum> for f64 {
    fn from(arg: Sum) -> Self {
        let one = f64::from_u128(u64::MAX as u128).unwrap();
        f64::from_u128(arg.0).unwrap() / one
    }
}
