// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cmp::Ordering,
    convert::{Into, TryFrom},
    fmt::Debug,
};

pub mod angle;

use crate::{
    hue::angle::Angle, proportion::Warmth, Chroma, ChromaOneRGB, HueConstants, LightLevel, Prop,
    RGBConstants, UFDRNumber, RGB,
};
use num_traits_plus::float_plus::FloatPlus;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct UFDRNumberRange {
    pub min: UFDRNumber,
    pub max_chroma_sum: UFDRNumber,
    pub max: UFDRNumber,
}

impl From<(UFDRNumber, UFDRNumber, UFDRNumber)> for UFDRNumberRange {
    fn from(tuple: (UFDRNumber, UFDRNumber, UFDRNumber)) -> Self {
        debug_assert!(tuple.0.is_hue_valid() && tuple.1.is_hue_valid() && tuple.2.is_hue_valid());
        debug_assert!(tuple.0 <= tuple.1 && tuple.1 <= tuple.2);
        Self {
            min: tuple.0,
            max_chroma_sum: tuple.1,
            max: tuple.2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum UFDRNumberOrdering {
    TooSmall(UFDRNumber),
    Shade(UFDRNumber, UFDRNumber),
    Neither(UFDRNumber),
    Tint(UFDRNumber, UFDRNumber),
    TooBig(UFDRNumber),
}

impl UFDRNumberOrdering {
    pub fn is_failure(&self) -> bool {
        use UFDRNumberOrdering::*;
        match self {
            TooSmall(_) | TooBig(_) => true,
            _ => false,
        }
    }

    pub fn is_success(&self) -> bool {
        use UFDRNumberOrdering::*;
        match self {
            TooSmall(_) | TooBig(_) => false,
            _ => true,
        }
    }
}

impl UFDRNumberRange {
    pub fn compare_sum(&self, sum: UFDRNumber) -> UFDRNumberOrdering {
        if sum < self.min {
            UFDRNumberOrdering::TooSmall(self.min - sum)
        } else if sum < self.max_chroma_sum - UFDRNumber(1) {
            UFDRNumberOrdering::Shade(self.min, self.max_chroma_sum - UFDRNumber(2))
        } else if sum > self.max_chroma_sum + UFDRNumber(1) {
            if sum <= self.max {
                UFDRNumberOrdering::Tint(self.max_chroma_sum + UFDRNumber(2), self.max)
            } else {
                UFDRNumberOrdering::TooBig(sum - self.max)
            }
        } else {
            UFDRNumberOrdering::Neither(self.max_chroma_sum)
        }
    }

    pub fn shade_min(&self) -> UFDRNumber {
        self.min
    }

    pub fn shade_max(&self) -> UFDRNumber {
        self.max_chroma_sum
    }

    pub fn crossover(&self) -> UFDRNumber {
        self.max_chroma_sum
    }

    pub fn tint_min(&self) -> UFDRNumber {
        self.max_chroma_sum
    }

    pub fn tint_max(&self) -> UFDRNumber {
        self.max
    }
}

pub trait HueIfce {
    fn angle(&self) -> Angle;
    fn sum_range_for_chroma_prop(&self, prop: Prop) -> Option<UFDRNumberRange>;
    fn sum_for_max_chroma(&self) -> UFDRNumber;
    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma>;
    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth;

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T>;
    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>>;
    fn min_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T>;
    fn max_sum_rgb_for_chroma<T: LightLevel>(&self, chroma: Chroma) -> RGB<T>;
    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>>;
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

impl ChromaOneRGB for RGBHue {
    /// RGB wih chroma of 1.0 chroma and with its hue (value may change op or down)
    fn chroma_one_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }
}

impl HueIfce for RGBHue {
    fn angle(&self) -> Angle {
        match self {
            RGBHue::Red => Angle::RED,
            RGBHue::Green => Angle::GREEN,
            RGBHue::Blue => Angle::BLUE,
        }
    }

    fn sum_range_for_chroma_prop(&self, prop: Prop) -> Option<UFDRNumberRange> {
        match prop {
            Prop::ZERO => None,
            Prop::ONE => Some(UFDRNumberRange::from((
                UFDRNumber::ONE,
                UFDRNumber::ONE,
                UFDRNumber::ONE,
            ))),
            prop => Some(UFDRNumberRange::from((
                prop.into(),
                UFDRNumber::ONE,
                (UFDRNumber::THREE - prop * 2),
            ))),
        }
    }

    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::ONE
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else if sum < UFDRNumber::ONE {
            Some(Chroma::Shade(sum.into()))
        } else if sum > UFDRNumber::ONE {
            Some(Chroma::Tint((UFDRNumber::THREE - sum) / 2))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        let x_dash = match self {
            RGBHue::Red => (UFDRNumber::ONE + chroma.prop()) / 2,
            RGBHue::Green | RGBHue::Blue => (UFDRNumber::TWO - chroma.prop()) / 4,
        };
        Warmth::calculate(chroma, x_dash)
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            RGBHue::Red => RGB::RED,
            RGBHue::Green => RGB::GREEN,
            RGBHue::Blue => RGB::BLUE,
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else {
            if sum <= UFDRNumber::ONE {
                Some(self.make_rgb((sum.into(), Prop::ZERO)))
            } else {
                Some(self.make_rgb((Prop::ONE, ((sum - UFDRNumber::ONE) / 2))))
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

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        match chroma.prop() {
            Prop::ZERO => None,
            c_prop => match sum.cmp(&c_prop.into()) {
                Ordering::Less => None,
                Ordering::Equal => Some(self.make_rgb::<T>((c_prop, Prop::ZERO))),
                Ordering::Greater => {
                    // NB: adjusting for rounding errors is proving elusive so we take the easiest
                    // option of having accurate chroma and up to 2 least significant errors in
                    // sum for the generated RGB (but we can adjust the UFDRNumber test to avoid unnecessary
                    // None returns.
                    let other = (sum - c_prop) / 3;
                    let first = other + c_prop;
                    // NB: Need to check that UFDRNumber wasn't too big
                    if first <= UFDRNumber::ONE {
                        assert_eq!(first.0 as u64 - other.0, c_prop.0);
                        assert!(sum.abs_diff(&(first + other * 2)) < UFDRNumber(3));
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

impl HueIfce for CMYHue {
    fn angle(&self) -> Angle {
        match self {
            CMYHue::Cyan => Angle::CYAN,
            CMYHue::Magenta => Angle::MAGENTA,
            CMYHue::Yellow => Angle::YELLOW,
        }
    }

    fn sum_range_for_chroma_prop(&self, prop: Prop) -> Option<UFDRNumberRange> {
        match prop {
            Prop::ZERO => None,
            Prop::ONE => Some(UFDRNumberRange::from((
                UFDRNumber::TWO,
                UFDRNumber::TWO,
                UFDRNumber::TWO,
            ))),
            prop => Some(UFDRNumberRange::from((
                prop * 2,
                UFDRNumber::TWO,
                UFDRNumber::THREE - prop,
            ))),
        }
    }

    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::TWO
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else if sum < UFDRNumber::TWO {
            Some(Chroma::Shade(sum / 2))
        } else if sum > UFDRNumber::TWO {
            Some(Chroma::Tint((UFDRNumber::THREE - sum).into()))
        } else {
            Some(Chroma::ONE)
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        let x_dash = match self {
            CMYHue::Cyan => (UFDRNumber::ONE - chroma.prop()) / 2,
            CMYHue::Magenta | CMYHue::Yellow => (UFDRNumber::TWO + chroma.prop()) / 4,
        };
        Warmth::calculate(chroma, x_dash)
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        match self {
            CMYHue::Cyan => RGB::CYAN,
            CMYHue::Magenta => RGB::MAGENTA,
            CMYHue::Yellow => RGB::YELLOW,
        }
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else if sum < UFDRNumber::TWO {
            Some(self.make_rgb(((sum / 2), Prop::ZERO)))
        } else if sum > UFDRNumber::TWO {
            Some(self.make_rgb((Prop::ONE, (sum - UFDRNumber::TWO).into())))
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

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        let sum_range = self.sum_range_for_chroma_prop(chroma.prop())?;
        match sum_range.compare_sum(sum) {
            UFDRNumberOrdering::TooSmall(_) | UFDRNumberOrdering::TooBig(_) => None,
            UFDRNumberOrdering::Neither(_) => Some(self.make_rgb((chroma.prop(), Prop::ZERO))),
            _ => Some(self.make_rgb(((sum + chroma.prop()) / 3, (sum - chroma.prop() * 2) / 3))),
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

    fn make_rgb_sum<T: LightLevel>(
        &self,
        components: (UFDRNumber, UFDRNumber, UFDRNumber),
    ) -> RGB<T> {
        debug_assert!(
            components.0 <= UFDRNumber::ONE
                && components.1 <= UFDRNumber::ONE
                && components.2 <= UFDRNumber::ONE
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

    pub fn sextant(&self) -> Sextant {
        self.0
    }

    pub fn prop(&self) -> Prop {
        self.1
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

impl HueIfce for SextantHue {
    fn angle(&self) -> Angle {
        let second: f64 = self.1.into();
        let sin = f64::SQRT_3 * second / 2.0 / (1.0 - second + second.powi(2)).sqrt();
        let angle = Angle::asin(Prop::from(sin));
        match self.0 {
            Sextant::RedMagenta => -angle,
            Sextant::RedYellow => angle,
            Sextant::GreenYellow => Angle::GREEN - angle,
            Sextant::GreenCyan => Angle::GREEN + angle,
            Sextant::BlueCyan => Angle::BLUE - angle,
            Sextant::BlueMagenta => Angle::BLUE + angle,
        }
    }

    fn sum_range_for_chroma_prop(&self, prop: Prop) -> Option<UFDRNumberRange> {
        match prop {
            Prop::ZERO => None,
            Prop::ONE => {
                let max_c_sum = Prop::ONE + self.1;
                Some(UFDRNumberRange::from((max_c_sum, max_c_sum, max_c_sum)))
            }
            prop => Some(UFDRNumberRange::from((
                (UFDRNumber::ONE + self.1) * prop,
                UFDRNumber::ONE + self.1,
                UFDRNumber::THREE - (UFDRNumber::TWO - self.1) * prop,
            ))),
        }
    }

    fn sum_for_max_chroma(&self) -> UFDRNumber {
        UFDRNumber::ONE + self.1
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        debug_assert!(sum.is_valid(), "sum: {:?}", sum);
        if sum == UFDRNumber::ZERO || sum == UFDRNumber::THREE {
            None
        } else {
            match sum.cmp(&(Prop::ONE + self.1)) {
                Ordering::Less => {
                    let temp = sum / (Prop::ONE + self.1);
                    Some(Chroma::Shade(temp))
                }
                Ordering::Greater => {
                    let temp = (UFDRNumber::THREE - sum) / (UFDRNumber::TWO - self.1);
                    Some(Chroma::Tint(temp))
                }
                Ordering::Equal => Some(Chroma::ONE),
            }
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
        let kc = chroma.prop() * self.1;
        let x_dash = match self.0 {
            // TODO: take tint and shade into account
            Sextant::RedYellow | Sextant::RedMagenta => {
                (UFDRNumber::TWO + chroma.prop() * 2 - kc) / 4
            }
            Sextant::GreenYellow | Sextant::BlueMagenta => {
                (UFDRNumber::TWO + kc * 2 - chroma.prop()) / 4
            }
            Sextant::GreenCyan | Sextant::BlueCyan => (UFDRNumber::TWO - kc - chroma.prop()) / 4,
        };
        Warmth::calculate(chroma, x_dash)
    }

    fn max_chroma_rgb<T: LightLevel>(&self) -> RGB<T> {
        self.make_rgb((Prop::ONE, self.1, Prop::ZERO))
    }

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
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

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
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
                        let three_delta = sum - ck_plus_c;
                        let delta = three_delta / 3;
                        let components = match three_delta % 3 {
                            // NB: allocation os spare light levels is done so as to preserve
                            // both the requested chroma and sum. Attempts to ensure hue does
                            // not drift have failed to rounding errors involved with division
                            1 => ((c_prop + delta), (ck + delta + Prop(1)), delta.into()),
                            2 => ((c_prop + delta + Prop(1)), (ck + delta), (delta + Prop(1))),
                            _ => ((c_prop + delta), (ck + delta), delta.into()),
                        };
                        debug_assert_eq!(components.0 + components.1 + components.2, sum);
                        debug_assert_eq!(components.0 - components.2, c_prop.into());
                        debug_assert!(
                            self.1
                                .abs_diff(
                                    (&((components.1 - components.2)
                                        / (components.0 - components.2)))
                                        .into()
                                )
                                .0
                                < 0x200
                        );
                        if components.0 <= UFDRNumber::ONE {
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

impl Default for Hue {
    fn default() -> Self {
        Self::RED
    }
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

impl From<Angle> for Hue {
    fn from(angle: Angle) -> Self {
        match angle {
            Angle::RED => Hue::RED,
            Angle::GREEN => Hue::GREEN,
            Angle::BLUE => Hue::BLUE,
            Angle::CYAN => Hue::CYAN,
            Angle::MAGENTA => Hue::MAGENTA,
            Angle::YELLOW => Hue::YELLOW,
            _ => {
                fn f(angle: Angle) -> Prop {
                    angle.sin() / (Angle::GREEN - angle).sin()
                };
                if angle >= Angle::RED {
                    if angle < Angle::YELLOW {
                        Hue::Sextant(SextantHue(Sextant::RedYellow, f(angle)))
                    } else if angle < Angle::GREEN {
                        Hue::Sextant(SextantHue(Sextant::GreenYellow, f(Angle::GREEN - angle)))
                    } else {
                        Hue::Sextant(SextantHue(Sextant::GreenCyan, f(angle - Angle::GREEN)))
                    }
                } else if angle > Angle::MAGENTA {
                    Hue::Sextant(SextantHue(Sextant::RedMagenta, f(-angle)))
                } else if angle > Angle::BLUE {
                    Hue::Sextant(SextantHue(Sextant::BlueMagenta, f(Angle::GREEN + angle)))
                } else {
                    Hue::Sextant(SextantHue(Sextant::BlueCyan, f(-angle - Angle::GREEN)))
                }
            }
        }
    }
}

impl HueIfce for Hue {
    fn angle(&self) -> Angle {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.angle(),
            Self::Secondary(cmy_hue) => cmy_hue.angle(),
            Self::Sextant(sextant_hue) => sextant_hue.angle(),
        }
    }

    fn sum_range_for_chroma_prop(&self, prop: Prop) -> Option<UFDRNumberRange> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_range_for_chroma_prop(prop),
            Self::Secondary(cmy_hue) => cmy_hue.sum_range_for_chroma_prop(prop),
            Self::Sextant(sextant_hue) => sextant_hue.sum_range_for_chroma_prop(prop),
        }
    }

    fn sum_for_max_chroma(&self) -> UFDRNumber {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.sum_for_max_chroma(),
            Self::Secondary(cmy_hue) => cmy_hue.sum_for_max_chroma(),
            Self::Sextant(sextant_hue) => sextant_hue.sum_for_max_chroma(),
        }
    }

    fn max_chroma_for_sum(&self, sum: UFDRNumber) -> Option<Chroma> {
        match self {
            Self::Primary(rgb_hue) => rgb_hue.max_chroma_for_sum(sum),
            Self::Secondary(cmy_hue) => cmy_hue.max_chroma_for_sum(sum),
            Self::Sextant(sextant_hue) => sextant_hue.max_chroma_for_sum(sum),
        }
    }

    fn warmth_for_chroma(&self, chroma: Chroma) -> Warmth {
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

    fn max_chroma_rgb_for_sum<T: LightLevel>(&self, sum: UFDRNumber) -> Option<RGB<T>> {
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

    fn rgb_for_sum_and_chroma<T: LightLevel>(
        &self,
        sum: UFDRNumber,
        chroma: Chroma,
    ) -> Option<RGB<T>> {
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
