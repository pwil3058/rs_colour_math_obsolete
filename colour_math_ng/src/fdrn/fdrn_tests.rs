// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use num_traits_plus::{assert_approx_eq, float_plus::FloatApproxEq};

use crate::UFDRNumber;

#[test]
fn ufdrn_div() {
    for [lhs, rhs] in &[
        [1.1_f64, 3.0],
        [0.0, 0.3],
        [1.024, 0.5],
        [0.18, 0.5],
        //[3000.0, 0.02],
        //[1.0, 1.001],
        //[25.0, 5.0],
    ] {
        let expected = UFDRNumber::from(lhs / rhs);
        println!("{:?} / {:?} = {:?} {:?}", lhs, rhs, lhs / rhs, expected);
        let result = UFDRNumber::from(*lhs) / UFDRNumber::from(*rhs);
        println!(
            "{:?} : {:?} diff = {:#X}",
            result,
            expected,
            result.abs_diff(&expected).0
        );
        assert_approx_eq!(result, expected, 0x4);
    }
}

#[test]
fn ufdrn_add() {
    for [a, b] in &[
        [0.0f64, 0.3],
        [0.024, 0.5],
        [0.18, 0.5],
        [0.5, 0.8],
        [1.5, 0.6],
    ] {
        let expected = UFDRNumber::from(a + b);
        println!(
            "{:?} + {:?} = {:?} {:?}",
            a,
            b,
            UFDRNumber::from(a + b),
            expected
        );
        let result = UFDRNumber::from(*a) + UFDRNumber::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x801);
        assert_approx_eq!(f64::from(result), &(a + b), 0.000_000_01);
    }
}

#[test]
fn ufdrn_sub() {
    for [a, b] in &[
        [0.5f64, 0.3],
        [0.524, 0.5],
        [0.18, 0.15],
        [0.5, 0.08],
        [1.2, 1.1],
    ] {
        let expected = UFDRNumber::from(a - b);
        println!("{:?} - {:?} = {:?} {:?}", a, b, a - b, expected);
        let result = UFDRNumber::from(*a) - UFDRNumber::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 257);
        assert_approx_eq!(f64::from(result), &(a - b), 0.000_000_01);
    }
}

#[test]
fn ufdrn_div_u8() {
    for (a, b) in &[(0.9_f64, 3_u8), (0.6, 2), (0.3, 2)] {
        let expected = UFDRNumber::from(*a / *b as f64);
        println!("{:?} / {:?} = {:?} : {:?}", a, b, a / *b as f64, expected);
        let result = UFDRNumber::from(*a) / *b;
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x156);
        assert_approx_eq!(f64::from(result), &(a / *b as f64), 0.000_000_01);
    }
}
