// 数字类型枚举

use ordered_float::OrderedFloat;
use std::fmt;
use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::error::ErrorExt;
use crate::push_compile_error;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Constant {
    Number(Number),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Number {
    Integer(i64),
    Float(OrderedFloat<f64>),
    Fraction(Fraction),
}

// 分数结构体
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Fraction {
    pub numerator: i64,   // 分子
    pub denominator: i64, // 分母
}

impl Constant {
    /// 创建一个数字常量
    pub fn number(num: Number) -> Constant {
        Constant::Number(num)
    }

    /// 创建一个布尔常量
    pub fn boolean(boolean: bool) -> Constant {
        Constant::Boolean(boolean)
    }

    /// 便捷构造器，创建一个整数常量
    pub fn integer(num: i64) -> Constant {
        Constant::Number(Number::Integer(num))
    }

    /// 便捷构造器，创建一个浮点数常量
    pub fn float(num: f64) -> Constant {
        Constant::Number(Number::Float(OrderedFloat(num)))
    }

    /// 便捷构造器，创建一个分数常量
    pub fn fraction(numerator: i64, denominator: i64) -> Constant {
        Constant::Number(Number::Fraction(Fraction::new(numerator, denominator)))
    }
}

impl Fraction {
    // 初始化
    pub fn new(numerator: i64, denominator: i64) -> Fraction {
        if denominator == 0 {
            push_compile_error!(ErrorExt::semantic_error("分母不能为零", true));
        }

        let frac: Fraction = Fraction {
            numerator,
            denominator,
        };
        frac
    }

    // 化简分数
    pub fn simplify(&mut self) {
        if self.numerator == 0 {
            self.denominator = 1;
            return;
        }

        // 处理符号，保证分母为正
        if self.denominator < 0 {
            self.numerator = -self.numerator;
            return;
        }

        // 求最大公约数并化简
        let gcd = gcd(self.numerator.abs(), self.denominator.abs());
        self.numerator /= gcd;
        self.denominator /= gcd;
    }

    // 转换为浮点数
    pub fn to_float(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    // 检查是否为整数
    pub fn is_integer(&self) -> bool {
        self.denominator == 1
    }

    // 转换为整数（如果可能）
    pub fn to_integer(&self) -> Option<i64> {
        if self.is_integer() {
            Some(self.numerator)
        } else {
            None
        }
    }

    // 转换为 LateX
    pub fn to_latex(&self) -> String {
        format!("\\frac{{{}}}{{{}}}", self.numerator, self.denominator)
    }
}

// 求最大公约数
fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

// 求最小公倍数
fn lcm(a: i64, b: i64) -> i64 {
    (a * b).abs() / gcd(a.abs(), b.abs())
}

impl Number {
    // 创建整数
    pub fn integer(value: i64) -> Number {
        Number::Integer(value)
    }

    // 创建浮点数
    pub fn float(value: f64) -> Number {
        Number::Float(OrderedFloat(value))
    }

    // 创建分数
    pub fn fraction(numerator: i64, denominator: i64) -> Number {
        Number::Fraction(Fraction::new(numerator, denominator))
    }

    // 转换为浮点数
    pub fn to_float(&self) -> f64 {
        match self {
            Number::Integer(i) => *i as f64,
            Number::Float(f) => (*f).0,
            Number::Fraction(frac) => frac.to_float(),
        }
    }

    // 尝试转换为整数
    pub fn to_integer(&self) -> Option<i64> {
        match self {
            Number::Integer(i) => Some(*i),
            Number::Float(f) => {
                if f.fract() == 0.0 {
                    Some((*f).0 as i64)
                } else {
                    None
                }
            }
            Number::Fraction(frac) => frac.to_integer(),
        }
    }

    // 检查是否为零
    pub fn is_zero(&self) -> bool {
        match self {
            Number::Integer(i) => *i == 0,
            Number::Float(f) => (*f).0 == 0.0,
            Number::Fraction(frac) => frac.numerator == 0,
        }
    }

    // 检查是否为 1
    pub fn is_one(&self) -> bool {
        match self {
            Number::Integer(i) => *i == 1,
            Number::Float(f) => (*f).0 == 1.0,
            Number::Fraction(frac) => frac.numerator == frac.denominator,
        }
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Integer(i) => write!(f, "{}", i),
            Number::Float(fl) => write!(f, "{}", fl),
            Number::Fraction(frac) => write!(f, "{}", frac),
        }
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Number(num) => write!(f, "{}", num),
            Constant::Boolean(b) => write!(f, "{}", b),
        }
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        match (self, other) {
            (Number::Integer(i1), Number::Integer(i2)) => {
                if let Some(result) = i1.checked_add(i2) {
                    Number::Integer(result)
                } else {
                    Number::Float(OrderedFloat(i1 as f64 + i2 as f64))
                }
            }
            (Number::Integer(i1), Number::Float(f2)) => {
                Number::Float(OrderedFloat(i1 as f64 + f2.0))
            }
            (Number::Integer(i1), Number::Fraction(frac2)) => {
                Number::Fraction(add_fractions(&Fraction::new(i1, 1), &frac2))
            }
            (Number::Float(f1), Number::Integer(i2)) => Number::Float(f1 + i2 as f64),
            (Number::Float(f1), Number::Float(f2)) => Number::Float(f1 + f2),
            (Number::Float(f1), Number::Fraction(frac2)) => Number::Float(f1 + frac2.to_float()),
            (Number::Fraction(frac1), Number::Integer(i2)) => {
                Number::Fraction(add_fractions(&frac1, &Fraction::new(i2, 1)))
            }
            (Number::Fraction(frac1), Number::Float(f2)) => {
                Number::Float(OrderedFloat(frac1.to_float() + f2.0))
            }
            (Number::Fraction(frac1), Number::Fraction(frac2)) => {
                Number::Fraction(add_fractions(&frac1, &frac2))
            }
        }
    }
}

impl Sub for Number {
    type Output = Number;

    fn sub(self, other: Number) -> Number {
        match (self, other) {
            (Number::Integer(i1), Number::Integer(i2)) => Number::Integer(i1 - i2),
            (Number::Integer(i1), Number::Float(f2)) => {
                Number::Float(OrderedFloat(i1 as f64 - f2.0))
            }
            (Number::Integer(i1), Number::Fraction(frac2)) => {
                Number::Fraction(sub_fractions(&Fraction::new(i1, 1), &frac2))
            }
            (Number::Float(f1), Number::Integer(i2)) => Number::Float(f1 - i2 as f64),
            (Number::Float(f1), Number::Float(f2)) => Number::Float(f1 - f2),
            (Number::Float(f1), Number::Fraction(frac2)) => Number::Float(f1 - frac2.to_float()),
            (Number::Fraction(frac1), Number::Integer(i2)) => {
                Number::Fraction(sub_fractions(&frac1, &Fraction::new(i2, 1)))
            }
            (Number::Fraction(frac1), Number::Float(f2)) => {
                Number::Float(OrderedFloat(frac1.to_float() - f2.0))
            }
            (Number::Fraction(frac1), Number::Fraction(frac2)) => {
                Number::Fraction(sub_fractions(&frac1, &frac2))
            }
        }
    }
}

impl Mul for Number {
    type Output = Number;

    fn mul(self, other: Number) -> Number {
        match (self, other) {
            (Number::Integer(i1), Number::Integer(i2)) => {
                if let Some(result) = i1.checked_mul(i2) {
                    Number::Integer(result)
                } else {
                    Number::Float(OrderedFloat(i1 as f64 * i2 as f64))
                }
            }
            (Number::Integer(i1), Number::Float(f2)) => {
                Number::Float(OrderedFloat(i1 as f64 * f2.0))
            }
            (Number::Integer(i1), Number::Fraction(frac2)) => {
                let mut result = frac2.clone();
                result.numerator *= i1;
                result.simplify();
                Number::Fraction(result)
            }
            (Number::Float(f1), Number::Integer(i2)) => Number::Float(f1 * i2 as f64),
            (Number::Float(f1), Number::Float(f2)) => Number::Float(f1 * f2),
            (Number::Float(f1), Number::Fraction(frac2)) => Number::Float(f1 * frac2.to_float()),
            (Number::Fraction(frac1), Number::Integer(i2)) => {
                let mut result = frac1.clone();
                result.numerator *= i2;
                result.simplify();
                Number::Fraction(result)
            }
            (Number::Fraction(frac1), Number::Float(f2)) => {
                Number::Float(OrderedFloat(frac1.to_float() * f2.0))
            }
            (Number::Fraction(frac1), Number::Fraction(frac2)) => {
                Number::Fraction(mul_fractions(&frac1, &frac2))
            }
        }
    }
}

impl Div for Number {
    type Output = Number;

    fn div(self, other: Number) -> Number {
        match (self, other) {
            (Number::Integer(i1), Number::Integer(i2)) => {
                if i1 % i2 == 0 {
                    Number::Integer(i1 / i2)
                } else {
                    Number::Fraction(Fraction::new(i1, i2))
                }
            }
            (Number::Integer(i1), Number::Float(f2)) => {
                Number::Float(OrderedFloat(i1 as f64 / f2.0))
            }
            (Number::Integer(i1), Number::Fraction(frac2)) => {
                Number::Fraction(div_fractions(&Fraction::new(i1, 1), &frac2))
            }
            (Number::Float(f1), Number::Integer(i2)) => Number::Float(f1 / i2 as f64),
            (Number::Float(f1), Number::Float(f2)) => Number::Float(f1 / f2),
            (Number::Float(f1), Number::Fraction(frac2)) => Number::Float(f1 / frac2.to_float()),
            (Number::Fraction(frac1), Number::Integer(i2)) => {
                Number::Fraction(div_fractions(&frac1, &Fraction::new(i2, 1)))
            }
            (Number::Fraction(frac1), Number::Float(f2)) => {
                Number::Float(OrderedFloat(frac1.to_float() / f2.0))
            }
            (Number::Fraction(frac1), Number::Fraction(frac2)) => {
                Number::Fraction(div_fractions(&frac1, &frac2))
            }
        }
    }
}

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Number {
        match self {
            Number::Integer(i) => Number::Integer(-i),
            Number::Float(f) => Number::Float(-f),
            Number::Fraction(frac) => {
                Number::Fraction(Fraction::new(-frac.numerator, frac.denominator))
            }
        }
    }
}

impl Sum for Number {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Number::Integer(0), |a, b| a + b)
    }
}

impl Product for Number {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Number::Integer(1), |a, b| a * b)
    }
}

fn add_fractions(a: &Fraction, b: &Fraction) -> Fraction {
    let common_denom = lcm(a.denominator, b.denominator);
    let num_a = a.numerator * (common_denom / a.denominator);
    let num_b = b.numerator * (common_denom / b.denominator);
    Fraction::new(num_a + num_b, common_denom)
}

fn sub_fractions(a: &Fraction, b: &Fraction) -> Fraction {
    let common_denom = lcm(a.denominator, b.denominator);
    let num_a = a.numerator * (common_denom / a.denominator);
    let num_b = b.numerator * (common_denom / b.denominator);
    Fraction::new(num_a - num_b, common_denom)
}

fn div_fractions(a: &Fraction, b: &Fraction) -> Fraction {
    if b.numerator == 0 {
        push_compile_error!(ErrorExt::semantic_error("除以零错误", true));
    }
    let mut frac = Fraction::new(a.numerator * b.denominator, a.denominator * b.numerator);
    frac.simplify();
    frac
}

fn mul_fractions(a: &Fraction, b: &Fraction) -> Fraction {
    let mut frac = Fraction::new(a.numerator * b.numerator, a.denominator * b.denominator);
    frac.simplify();
    frac
}

impl PartialEq<i64> for Number {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Number::Integer(i) => i == other,
            Number::Float(f) => (*f).0 == *other as f64,
            Number::Fraction(frac) => frac.denominator == 1 && frac.numerator == *other,
        }
    }
}

impl PartialOrd<i64> for Number {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        match self {
            Number::Integer(i) => i.partial_cmp(other),
            Number::Float(f) => (*f).0.partial_cmp(&(*other as f64)),
            Number::Fraction(frac) => {
                (frac.numerator as f64 / frac.denominator as f64).partial_cmp(&(*other as f64))
            }
        }
    }
}
