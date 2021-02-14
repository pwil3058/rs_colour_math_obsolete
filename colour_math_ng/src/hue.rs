// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{Into, TryFrom},
    fmt::Debug,
};

use normalised_angles::Degrees;

use crate::{
    Chroma, ChromaOneRGB, Float, HueAngle, HueConstants, LightLevel, Prop, RGBConstants, Sum, RGB,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct SumRange((Sum, Sum, Sum));

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum SumOrdering {
    TooSmall(Sum),
    Shade(Sum, Sum),
    Tint(Sum, Sum),
    TooBig(Sum),
}

impl SumOrdering {
    pub fn is_failure(&self) -> bool {
        use SumOrdering::*;
        match self {
            TooSmall(_) | TooBig(_) => true,
            _ => false,
        }
    }

    pub fn is_success(&self) -> bool {
        use SumOrdering::*;
        match self {
            TooSmall(_) | TooBig(_) => false,
            _ => true,
        }
    }
}

impl SumRange {
    pub fn compare_sum(&self, sum: Sum) -> SumOrdering {
        if sum < self.0 .0 {
            SumOrdering::TooSmall(self.0 .0 - sum)
        } else if sum <= self.0 .1 {
            SumOrdering::Shade(self.0 .0, self.0 .1)
        } else if sum <= self.0 .2 {
            SumOrdering::Tint(self.0 .1, self.0 .2)
        } else {
            SumOrdering::TooBig(sum - self.0 .2)
        }
    }

    pub fn min(&self) -> Sum {
        self.0 .0
    }

    pub fn shade_min(&self) -> Sum {
        self.0 .0
    }

    pub fn shade_max(&self) -> Sum {
        self.0 .1
    }

    pub fn crossover(&self) -> Sum {
        self.0 .1
    }

    pub fn tint_min(&self) -> Sum {
        self.0 .1
    }

    pub fn tint_max(&self) -> Sum {
        self.0 .2
    }

    pub fn max(&self) -> Sum {
        self.0 .2
    }
}

pub trait HueIfce {
    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<SumRange>;
    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma>;
    fn warmth_for_chroma(&self, chroma: Chroma) -> Prop;

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T>;
    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>>;
    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T>;
    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T>;
    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Chroma) -> Option<RGB<T>>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RGBHue {
    Red = 5,
    Green = 9,
    Blue = 1,
}

impl RGBHue {
    fn make_rgb<T: LightLevel>(&self, components: (Prop, Prop)) -> RGB<T> {
        use RGBHue::*;
        match self {
            Red => [components.0, components.1, components.1].into(),
            Green => [components.1, components.0, components.1].into(),
            Blue => [components.1, components.1, components.0].into(),
        }
    }
}

impl<T: Float> HueAngle<T> for RGBHue {
    fn hue_angle(&self) -> Degrees<T> {
        match self {
            RGBHue::Red => Degrees::RED,
            RGBHue::Green => Degrees::GREEN,
            RGBHue::Blue => Degrees::BLUE,
        }
    }
}

impl<T: LightLevel> ChromaOneRGB<T> for RGBHue {
    /// RGB wih chroma of 1.0 chroma and with its hue (value may change op or down)
    fn chroma_one_rgb(&self) -> RGB<T> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }
}

impl HueIfce for RGBHue {
    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<SumRange> {
        match chroma.prop() {
            Prop::ZERO => None,
            prop => Some(SumRange((prop.into(), Sum::ONE, (Sum::THREE - prop * 2)))),
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::ONE {
            Some(Chroma::Shade(sum.into()))
        } else if sum > Sum::ONE {
            Some(Chroma::Tint((Sum::THREE - sum) / 2))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Prop {
        match self {
            // TODO: take tint and shade into account
            RGBHue::Red => (Sum::ONE + chroma.prop()) / 2,
            RGBHue::Green | RGBHue::Blue => (Sum::TWO - chroma.prop()) / 4,
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else {
            if sum <= Sum::ONE {
                Some(self.make_rgb((sum.into(), Prop::ZERO)))
            } else {
                Some(self.make_rgb((Prop::ONE, ((sum - Sum::ONE) / 2))))
            }
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::BLACK,
            Prop::ONE => self.max_chroma_rgb(),
            prop => self.make_rgb((prop, Prop::ZERO)),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::WHITE,
            Prop::ONE => self.max_chroma_rgb(),
            prop => self.make_rgb((Prop::ONE, Prop::ONE - prop)),
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Chroma) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        match chroma.prop() {
            Prop::ZERO => None,
            c_prop => match sum.cmp(&c_prop.into()) {
                Ordering::Less => None,
                Ordering::Equal => Some(self.make_rgb::<T>((c_prop, Prop::ZERO))),
                Ordering::Greater => {
                    // NB: adjusting for rounding errors is proving elusive so we take the easiest
                    // option of having accurate chroma and up to 2 least significant errors in
                    // sum for the generated RGB (but we can adjust the Sum test to avoid unnecessary
                    // None returns.
                    let other = (sum - c_prop) / 3;
                    let first = other + c_prop;
                    // NB: Need to check that Sum wasn't too big
                    if first <= Sum::ONE {
                        assert_eq!(first.0 as u64 - other.0, c_prop.0);
                        assert!(sum.abs_diff(&(first + other * 2)) < Sum(3));
                        Some(self.make_rgb::<T>((first.into(), other)))
                    } else {
                        None
                    }
                }
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CMYHue {
    Cyan = 113,
    Magenta = 3,
    Yellow = 7,
}

impl CMYHue {
    fn make_rgb<T: LightLevel>(&self, components: (Prop, Prop)) -> RGB<T> {
        use CMYHue::*;
        match self {
            Cyan => [components.1, components.0, components.0].into(),
            Magenta => [components.0, components.1, components.0].into(),
            Yellow => [components.0, components.0, components.1].into(),
        }
    }
}

impl<T: Float> HueAngle<T> for CMYHue {
    fn hue_angle(&self) -> Degrees<T> {
        match self {
            CMYHue::Cyan => Degrees::CYAN,
            CMYHue::Magenta => Degrees::MAGENTA,
            CMYHue::Yellow => Degrees::YELLOW,
        }
    }
}

impl HueIfce for CMYHue {
    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<SumRange> {
        if chroma.prop() == Prop::ZERO {
            None
        } else {
            Some(SumRange((
                chroma.prop() * 2,
                Sum::TWO,
                Sum::THREE - chroma.prop(),
            )))
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::TWO {
            Some(Chroma::Shade(sum / 2))
        } else if sum > Sum::TWO {
            Some(Chroma::Tint((Sum::THREE - sum).into()))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Prop {
        match self {
            // TODO: take tint and shade into account
            CMYHue::Cyan => (Sum::ONE - chroma.prop()) / 2,
            CMYHue::Magenta | CMYHue::Yellow => (Sum::TWO + chroma.prop()) / 4,
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            CMYHue::Cyan => RGB::CYAN,
            CMYHue::Magenta => RGB::MAGENTA,
            CMYHue::Yellow => RGB::YELLOW,
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else if sum < Sum::TWO {
            Some(self.make_rgb(((sum / 2), Prop::ZERO)))
        } else if sum > Sum::TWO {
            Some(self.make_rgb((Prop::ONE, (sum - Sum::TWO).into())))
        } else {
            Some(self.max_chroma_rgb())
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::BLACK,
            Prop::ONE => self.max_chroma_rgb(),
            prop => self.make_rgb((prop, Prop::ZERO)),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::WHITE,
            Prop::ONE => self.max_chroma_rgb(),
            prop => self.make_rgb((Prop::ONE, Prop::ONE - prop)),
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Chroma) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        let _dummy = match chroma.prop() {
            Prop::ZERO => None,
            c_prop => {
                let two_c = c_prop * 2;
                match sum.cmp(&two_c) {
                    Ordering::Less => None,
                    Ordering::Equal => Some(self.make_rgb::<T>((c_prop, Prop::ZERO))),
                    Ordering::Greater => {
                        // NB: adjusting for rounding errors is proving elusive so we take the easiest
                        // option of having accurate chroma and up to 2 least significant digit
                        // errors in sum for the generated RGB (but we can adjust the Sum test to
                        // avoid unnecessary None returns.
                        let other = (sum - two_c) / 3;
                        let primary = other + c_prop;
                        // NB: Need to check that Sum wasn't too big
                        if primary <= Sum::ONE {
                            assert_eq!(primary.0 as u64 - other.0, c_prop.0);
                            assert!(sum.abs_diff(&(primary + primary + other)) < Sum(3));
                            Some(self.make_rgb::<T>((primary.into(), other)))
                        } else {
                            None
                        }
                    }
                }
            }
        };
        let sum_range = self.sum_range_for_chroma(chroma)?;
        if sum_range.compare_sum(sum).is_success() {
            // TODO: reassess this calculation
            Some(self.make_rgb(((sum + chroma.prop()) / 3, (sum - chroma.prop() * 2) / 3)))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Sextant {
    RedMagenta = 4,
    RedYellow = 6,
    GreenYellow = 8,
    GreenCyan = 10,
    BlueCyan = 0,
    BlueMagenta = 2,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SextantHue(Sextant, Prop);

impl Eq for SextantHue {}

impl SextantHue {
    fn make_rgb<T: LightLevel>(&self, components: (Prop, Prop, Prop)) -> RGB<T> {
        use Sextant::*;
        match self.0 {
            RedMagenta => [components.0, components.2, components.1].into(),
            RedYellow => [components.0, components.1, components.2].into(),
            GreenYellow => [components.1, components.0, components.2].into(),
            GreenCyan => [components.2, components.0, components.1].into(),
            BlueCyan => [components.2, components.1, components.0].into(),
            BlueMagenta => [components.1, components.2, components.0].into(),
        }
    }

    fn make_rgb_sum<T: LightLevel>(&self, components: (Sum, Sum, Sum)) -> RGB<T> {
        debug_assert!(
            components.0 <= Sum::ONE && components.1 <= Sum::ONE && components.2 <= Sum::ONE
        );
        self.make_rgb((
            components.0.into(),
            components.1.into(),
            components.2.into(),
        ))
    }

    pub fn abs_diff(&self, other: &Self) -> Prop {
        if self.0 == other.0 {
            self.1.abs_diff(&other.1)
        } else {
            Prop::ONE
        }
    }
}

#[cfg(test)]
impl SextantHue {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        if self.0 == other.0 {
            self.1.approx_eq(&other.1, acceptable_rounding_error)
        } else {
            false
        }
    }
}

impl<T: LightLevel> From<(Sextant, &RGB<T>)> for SextantHue {
    fn from(arg: (Sextant, &RGB<T>)) -> Self {
        use Sextant::*;
        let [red, green, blue] = <[Prop; 3]>::from(*arg.1);
        let other = match arg.0 {
            RedMagenta => (blue - green) / (red - green),
            RedYellow => (green - blue) / (red - blue),
            GreenYellow => (red - blue) / (green - blue),
            GreenCyan => (blue - red) / (green - red),
            BlueCyan => (green - red) / (blue - red),
            BlueMagenta => (red - green) / (blue - green),
        };
        debug_assert!(other > Prop::ZERO && other < Prop::ONE);
        Self(arg.0, other)
    }
}

impl<T: Float + From<Prop> + Copy> HueAngle<T> for SextantHue {
    fn hue_angle(&self) -> Degrees<T> {
        let second: T = self.1.into();
        let sin = T::SQRT_3 * second / T::TWO / (T::ONE - second + second.powi(2)).sqrt();
        let angle = Degrees::asin(sin);
        match self.0 {
            Sextant::RedMagenta => -angle,
            Sextant::RedYellow => angle,
            Sextant::GreenYellow => Degrees::GREEN - angle,
            Sextant::GreenCyan => Degrees::GREEN + angle,
            Sextant::BlueCyan => Degrees::BLUE - angle,
            Sextant::BlueMagenta => Degrees::BLUE + angle,
        }
    }
}

impl HueIfce for SextantHue {
    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<SumRange> {
        match chroma.prop() {
            Prop::ZERO => None,
            Prop::ONE => {
                let max_c_sum = Prop::ONE + self.1;
                Some(SumRange((max_c_sum, max_c_sum, max_c_sum)))
            }
            chroma_p => {
                let ck = self.1 * chroma_p;
                Some(SumRange((
                    chroma_p + ck,
                    Sum::ONE + self.1,
                    Sum::THREE - chroma_p * 2 + ck,
                )))
            }
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == Sum::ZERO || sum == Sum::THREE {
            None
        } else {
            match sum.cmp(&(Prop::ONE + self.1)) {
                Ordering::Less => {
                    let temp = sum / (Prop::ONE + self.1);
                    Some(Chroma::Shade(temp))
                }
                Ordering::Greater => {
                    let temp = (Sum::THREE - sum) / (Sum::TWO - self.1);
                    Some(Chroma::Tint(temp))
                }
                Ordering::Equal => Some(Chroma::ONE),
            }
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Prop {
        let kc = chroma.prop() * self.1;
        match self.0 {
            // TODO: take tint and shade into account
            Sextant::RedYellow | Sextant::RedMagenta => (Sum::TWO + chroma.prop() * 2 - kc) / 4,
            Sextant::GreenYellow | Sextant::BlueMagenta => (Sum::TWO + kc * 2 - chroma.prop()) / 4,
            Sextant::GreenCyan | Sextant::BlueCyan => (Sum::TWO - kc - chroma.prop()) / 4,
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        self.make_rgb((Prop::ONE, self.1, Prop::ZERO))
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        let chroma = self.max_chroma_for_sum(sum)?;
        Some(self.rgb_for_sum_and_chroma(sum, chroma)?)
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::BLACK,
            Prop::ONE => self.max_chroma_rgb(),
            c_prop => self.make_rgb((c_prop, self.1 * c_prop, Prop::ZERO)),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match chroma.prop() {
            Prop::ZERO => RGB::<T>::WHITE,
            Prop::ONE => self.max_chroma_rgb(),
            c_prop => {
                let third = Prop::ONE - c_prop;
                let second = c_prop * self.1 + third;
                self.make_rgb((Prop::ONE, second.into(), third))
            }
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Chroma) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        match chroma.prop() {
            Prop::ZERO => None,
            c_prop => {
                let ck = self.1 * c_prop;
                let ck_plus_c = ck + c_prop;
                match sum.cmp(&ck_plus_c) {
                    Ordering::Less => None,
                    Ordering::Equal => Some(self.make_rgb((c_prop, ck, Prop::ZERO))),
                    Ordering::Greater => {
                        let three_first = sum + c_prop * 2 - ck;
                        if three_first <= Sum::ONE {
                            let first = three_first / 3;
                            let three_second = sum + ck * 2 - c_prop;
                            let second = three_second / 3;
                            if three_second > three_first * self.1 {
                                let three_third: Sum = ((three_second - three_first * self.1)
                                    / (Sum::ONE - self.1))
                                    .into();
                                let third = three_third / 3;
                                let c_out = first - third;
                                let sum_out = first + second + third;
                                let k_out: Prop = ((second - third) / (first - third)).into();
                                println!(
                                "ALT: Hue diff: {:#X} {:?} {:?} Chroma diff: {:#X} Sum diff: {:#X}",
                                self.1.abs_diff(&k_out).0,
                                k_out > self.1,
                                k_out < self.1,
                                c_out.abs_diff(&c_prop).0,
                                sum_out.abs_diff(&sum).0
                            );
                                //debug_assert_eq!(c_out, c_prop,);
                                //debug_assert_eq!(sum_out, sum,);
                                //debug_assert_eq!(k_out, self.1,);
                            }
                        };
                        let three_delta = sum - ck_plus_c;
                        let delta = three_delta / 3;
                        let components = match three_delta.0 % 3 {
                            // NB: allocation os spare light levels is done so as to preserve
                            // both the requested chroma and sum
                            1 => ((c_prop + delta), (ck + delta + Prop(1)), delta.into()),
                            2 => ((c_prop + delta + Prop(1)), (ck + delta), (delta + Prop(1))),
                            _ => ((c_prop + delta), (ck + delta), delta.into()),
                        };
                        debug_assert_eq!(components.0 + components.1 + components.2, sum);
                        debug_assert_eq!(components.0 - components.2, c_prop.into());
                        let k_out = (components.1 - components.2) / (components.0 - components.2);
                        println!(
                            "ABS DIFF: {:#X} {:?}",
                            self.1.abs_diff(&k_out).0,
                            k_out > self.1
                        );
                        // debug_assert!(
                        //     self.1.abs_diff(
                        //         &((components.1 - components.2) / (components.0 - components.2))
                        //     ) < Prop(0x2F)
                        // );
                        if components.0 <= Sum::ONE {
                            Some(self.make_rgb_sum::<T>(components))
                        } else {
                            None
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Hue {
    Primary(RGBHue),
    Secondary(CMYHue),
    Sextant(SextantHue),
}

impl Eq for Hue {}

impl HueConstants for Hue {
    const RED: Self = Self::Primary(RGBHue::Red);
    const GREEN: Self = Self::Primary(RGBHue::Green);
    const BLUE: Self = Self::Primary(RGBHue::Blue);

    const CYAN: Self = Self::Secondary(CMYHue::Cyan);
    const MAGENTA: Self = Self::Secondary(CMYHue::Magenta);
    const YELLOW: Self = Self::Secondary(CMYHue::Yellow);
}

impl<T: LightLevel> TryFrom<&RGB<T>> for Hue {
    type Error = &'static str;

    fn try_from(rgb: &RGB<T>) -> Result<Self, Self::Error> {
        use Sextant::*;
        let [red, green, blue] = <[Prop; 3]>::from(*rgb);
        match red.cmp(&green) {
            Ordering::Greater => match green.cmp(&blue) {
                Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((RedYellow, rgb)))),
                Ordering::Less => match red.cmp(&blue) {
                    Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((RedMagenta, rgb)))),
                    Ordering::Less => Ok(Hue::Sextant(SextantHue::from((BlueMagenta, rgb)))),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Magenta)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Red)),
            },
            Ordering::Less => match red.cmp(&blue) {
                Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((GreenYellow, rgb)))),
                Ordering::Less => match green.cmp(&blue) {
                    Ordering::Greater => Ok(Hue::Sextant(SextantHue::from((GreenCyan, rgb)))),
                    Ordering::Less => Ok(Hue::Sextant(SextantHue::from((BlueCyan, rgb)))),
                    Ordering::Equal => Ok(Hue::Secondary(CMYHue::Cyan)),
                },
                Ordering::Equal => Ok(Hue::Primary(RGBHue::Green)),
            },
            Ordering::Equal => match red.cmp(&blue) {
                Ordering::Greater => Ok(Hue::Secondary(CMYHue::Yellow)),
                Ordering::Less => Ok(Hue::Primary(RGBHue::Blue)),
                Ordering::Equal => Err("RGB is grey and hs no hue"),
            },
        }
    }
}

impl<T: Float + From<Prop>> HueAngle<T> for Hue {
    fn hue_angle(&self) -> Degrees<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.hue_angle(),
            Self::Secondary(cmy_hue) => cmy_hue.hue_angle(),
            Self::Sextant(sextant_hue) => sextant_hue.hue_angle(),
        }
    }
}

impl HueIfce for Hue {
    fn sum_range_for_chroma(&self, chroma: Chroma) -> Option<SumRange> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_range_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.sum_range_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma(chroma),
        }
    }

    fn max_chroma_for_sum(&self, sum: Sum) -> Option<Chroma> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_for_sum(sum),
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Prop {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.warmth_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.warmth_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.warmth_for_chroma(chroma),
        }
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb(),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb(),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb(),
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: Sum) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_rgb_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_rgb_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_rgb_for_sum(sum),
        }
    }

    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.min_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.min_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.min_sum_rgb_for_chroma(chroma),
        }
    }

    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_sum_rgb_for_chroma(chroma),
            Self::Secondary(cmy_hue) => cmy_hue.max_sum_rgb_for_chroma(chroma),
            Self::Sextant(sextant_hue) => sextant_hue.max_sum_rgb_for_chroma(chroma),
        }
    }

    fn rgb_for_sum_and_chroma<T: LightLevel>(&self, sum: Sum, chroma: Chroma) -> Option<RGB<T>> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Secondary(cmy_hue) => cmy_hue.rgb_for_sum_and_chroma(sum, chroma),
            Self::Sextant(sextant_hue) => sextant_hue.rgb_for_sum_and_chroma(sum, chroma),
        }
    }
}

impl Hue {
    pub fn ord_index(&self) -> u8 {
        0
    }

    pub fn abs_diff(&self, other: &Self) -> Prop {
        match self {
            Self::Primary(rgb_hue) => match other {
                Self::Primary(other_rgb_hue) => {
                    if rgb_hue == other_rgb_hue {
                        Prop::ZERO
                    } else {
                        Prop::ONE
                    }
                }
                _ => Prop::ONE,
            },
            Self::Secondary(cmy_hue) => match other {
                Self::Secondary(other_cmy_hue) => {
                    if cmy_hue == other_cmy_hue {
                        Prop::ZERO
                    } else {
                        Prop::ONE
                    }
                }
                _ => Prop::ONE,
            },
            Self::Sextant(sextant_hue) => match other {
                Self::Sextant(other_sextant_hue) => sextant_hue.1.abs_diff(&other_sextant_hue.1),
                _ => Prop::ONE,
            },
        }
    }
}

#[cfg(test)]
impl Hue {
    pub fn approx_eq(&self, other: &Self, acceptable_rounding_error: Option<u64>) -> bool {
        match self {
            Self::Primary(rgb_hue) => match other {
                Self::Primary(other_rgb_hue) => rgb_hue == other_rgb_hue,
                _ => false,
            },
            Self::Secondary(cmy_hue) => match other {
                Self::Secondary(other_cmy_hue) => cmy_hue == other_cmy_hue,
                _ => false,
            },
            Self::Sextant(sextant_hue) => match other {
                Self::Sextant(other_sextant_hue) => {
                    sextant_hue.approx_eq(other_sextant_hue, acceptable_rounding_error)
                }
                _ => false,
            },
        }
    }
}

impl PartialOrd for Hue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ord_index().partial_cmp(&other.ord_index())
    }
}

impl Ord for Hue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod hue_tests;
