#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

use std::{cmp::PartialEq, fmt::Debug};

use num_traits_plus;
use num_traits_plus::float_plus::*;

pub use normalised_angles::{Degrees, DegreesConst, RadiansConst};

pub mod attributes;
pub mod chroma;
pub mod hcv;
pub mod hue;
pub mod image;
pub mod manipulator;
pub mod rgb;
pub mod rgba;
pub mod urgb;
pub mod urgba;

pub use crate::hcv::HCV;
pub use crate::hue::Hue;
pub use crate::rgb::{IndicesValueOrder, RGB};
pub use crate::rgba::RGBA;
pub use crate::urgb::{RGB16, RGB8, URGB};
pub use crate::urgba::{RGBA16, RGBA8, URGBA};

pub trait HueConstants: Sized + Copy {
    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;

    const CYAN: Self;
    const MAGENTA: Self;
    const YELLOW: Self;

    const PRIMARIES: [Self; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
}

pub trait RGBConstants: HueConstants + Copy {
    const WHITE: Self;
    const BLACK: Self;

    const GREYS: [Self; 2] = [Self::BLACK, Self::WHITE];
}

impl<F: FloatPlus + DegreesConst + Debug> HueConstants for Degrees<F> {
    const RED: Self = Self::DEG_0;
    const GREEN: Self = Self::DEG_120;
    const BLUE: Self = Self::NEG_DEG_120;

    const CYAN: Self = Self::DEG_180;
    const MAGENTA: Self = Self::NEG_DEG_60;
    const YELLOW: Self = Self::DEG_60;
}

pub trait ColourComponent:
    FloatPlus + DegreesConst + RadiansConst + std::iter::Sum + Debug + Default
{
    const FOUR: Self;
    const SIN_120: Self;
    const COS_120: Self;

    fn is_proportion(self) -> bool {
        self <= Self::ONE && self >= Self::ZERO
    }
}

impl ColourComponent for f32 {
    const FOUR: Self = 4.0;
    const SIN_120: Self = 0.86602_54;
    const COS_120: Self = -0.5;
}

impl ColourComponent for f64 {
    const FOUR: Self = 4.0;
    const SIN_120: Self = 0.86602_54037_84439;
    const COS_120: Self = -0.5;
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CCI {
    Red,
    Green,
    Blue,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ScalarAttribute {
    Chroma,
    Greyness,
    Value,
    Warmth,
}

impl std::fmt::Display for ScalarAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ScalarAttribute::Chroma => write!(f, "Chroma"),
            ScalarAttribute::Greyness => write!(f, "Greyness"),
            ScalarAttribute::Value => write!(f, "Value"),
            ScalarAttribute::Warmth => write!(f, "Warmth"),
        }
    }
}

pub trait HueIfce<F: ColourComponent> {
    fn hue_angle(&self) -> Degrees<F>;
    fn chroma_correction(&self) -> F;
    fn sum_range_for_chroma(&self, chroma: F) -> (F, F);
    fn max_chroma_for_sum(&self, sum: F) -> F;

    fn max_chroma_rgb(&self) -> RGB<F>;
    fn max_chroma_rgb_for_sum(&self, sum: F) -> RGB<F>;
    fn min_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F>;
    fn max_sum_rgb_for_chroma(&self, chroma: F) -> RGB<F>;
    fn rgb_for_sum_and_chroma(&self, sum: F, chroma: F) -> Option<RGB<F>>;

    fn value_range_for_chroma(&self, chroma: F) -> (F, F) {
        let (min, max) = self.sum_range_for_chroma(chroma);
        ((min / F::THREE).min(F::ONE), (max / F::THREE).min(F::ONE))
    }

    fn max_chroma_for_value(&self, value: F) -> F {
        self.max_chroma_for_sum(value * F::THREE)
    }

    fn max_chroma_rgb_for_value(&self, value: F) -> RGB<F> {
        self.max_chroma_rgb_for_sum(value * F::THREE)
    }

    fn rgb_for_value_and_chroma(&self, value: F, chroma: F) -> Option<RGB<F>> {
        self.rgb_for_sum_and_chroma(value * F::THREE, chroma)
    }
}

pub trait ColourInterface<F: ColourComponent> {
    fn rgb(&self) -> RGB<F>;

    fn rgba(&self) -> RGBA<F>;

    fn hcv(&self) -> HCV<F>;

    fn hue(&self) -> Option<Hue<F>>;

    fn hue_angle(&self) -> Option<Degrees<F>>;

    fn is_grey(&self) -> bool;

    fn chroma(&self) -> F;

    fn greyness(&self) -> F;

    fn value(&self) -> F;

    fn warmth(&self) -> F;

    fn best_foreground_rgb(&self) -> RGB<F>;

    fn monochrome_rgb(&self) -> RGB<F>;

    fn max_chroma_rgb(&self) -> RGB<F>;

    fn warmth_rgb(&self) -> RGB<F>;

    fn scalar_attribute(&self, attr: ScalarAttribute) -> F {
        match attr {
            ScalarAttribute::Chroma => self.chroma(),
            ScalarAttribute::Greyness => self.greyness(),
            ScalarAttribute::Value => self.value(),
            ScalarAttribute::Warmth => self.warmth(),
        }
    }

    fn scalar_attribute_rgb(&self, attr: ScalarAttribute) -> RGB<F> {
        match attr {
            ScalarAttribute::Chroma => self.rgb(),
            ScalarAttribute::Greyness => self.rgb(),
            ScalarAttribute::Value => self.monochrome_rgb(),
            ScalarAttribute::Warmth => self.warmth_rgb(),
        }
    }
}
