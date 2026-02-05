use core::fmt;
use std::{fmt::Formatter, slice::IterMut, vec::IntoIter};

use crate::{
    lexer::constant::Number,
    semantic::{semantic_ir::numeric::NumericExpression, variable::Variable},
};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct NumericBucket {
    pub constants: Vec<Number>,
    pub variables: Vec<Variable>,
    pub expressions: Vec<NumericExpression>,
}

// 自定义不可变迭代器
// state = 0: 常量
// state = 1: 变量
// state = 2: 表达式
pub struct NumericBucketIter<'a> {
    bucket: &'a NumericBucket,
    state: u8,
    index: usize,
}

// 可变迭代器：拥有对底层三组集合的迭代器
pub struct NumericBucketIterMut<'a> {
    constants: IterMut<'a, Number>,
    variables: IterMut<'a, Variable>,
    expressions: IterMut<'a, NumericExpression>,
    state: u8,
}

// IntoIter（获取所有者的迭代器）
pub struct NumericBucketIntoIter {
    constants: IntoIter<Number>,
    variables: IntoIter<Variable>,
    expressions: IntoIter<NumericExpression>,
    state: u8,
}

// 可变项的视图：因为常量与变量存储为单独类型而不是 `NumericExpression`，
// 我们在可变迭代中返回一个包含对底层元素可变引用的枚举。
pub enum NumericExpressionMut<'a> {
    Constant(&'a mut Number),
    Variable(&'a mut Variable),
    Expression(&'a mut NumericExpression),
}

impl NumericBucket {
    pub fn new() -> Self {
        NumericBucket {
            constants: Vec::new(),
            variables: Vec::new(),
            expressions: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.constants.len() + self.variables.len() + self.expressions.len()
    }

    pub fn push(&mut self, expr: NumericExpression) {
        match expr {
            NumericExpression::Constant(c) => self.constants.push(c),
            NumericExpression::Variable(v) => self.variables.push(v),
            _ => self.expressions.push(expr),
        }
    }

    pub fn extend(&mut self, other: NumericBucket) {
        self.constants.extend(other.constants);
        self.variables.extend(other.variables);
        self.expressions.extend(other.expressions);
    }

    pub fn execute_constant(&mut self, add: bool) {
        if add {
            let sum: Number = self.constants.iter().cloned().sum();
            self.constants.clear();
            self.constants.push(sum);
        } else {
            let product: Number = self.constants.iter().cloned().product();
            self.constants.clear();
            self.constants.push(product);
        }
    }

    pub fn iter(&'_ self) -> NumericBucketIter<'_> {
        NumericBucketIter {
            bucket: self,
            state: 0,
            index: 0,
        }
    }

    // 新增：可变迭代器
    pub fn iter_mut(&'_ mut self) -> NumericBucketIterMut<'_> {
        NumericBucketIterMut {
            constants: self.constants.iter_mut(),
            variables: self.variables.iter_mut(),
            expressions: self.expressions.iter_mut(),
            state: 0,
        }
    }

    pub fn contains_one(&self) -> bool {
        if self.constants.len() == 1 && self.constants[0].is_one() {
            return true;
        } else if self.constants.len() > 1 {
            panic!("must execute constant before detect.");
        }
        false
    }

    pub fn contains_zero(&self) -> bool {
        if self.constants.len() == 1 && self.constants[0].is_zero() {
            return true;
        } else if self.constants.len() > 1 {
            panic!("must execute constant before detect.");
        }
        false
    }

    pub fn remove_one(&mut self) {
        if self.contains_one() {
            self.constants.remove(0);
        }
    }

    pub fn remove_zero(&mut self) {
        if self.contains_zero() {
            self.constants.remove(0);
        }
    }

    pub fn contains_constant(&self) -> bool {
        self.constants.len() > 0
    }

    pub fn get_constants(&self) -> Vec<Number> {
        self.constants.clone()
    }

    pub fn get_non_constants(&self) -> Vec<NumericExpression> {
        let mut vars: Vec<NumericExpression> = self
            .variables
            .clone()
            .into_iter()
            .map(NumericExpression::variable)
            .collect();
        let mut exprs: Vec<NumericExpression> = self.expressions.clone();
        exprs.append(&mut vars);
        exprs
    }

    pub fn without_constants(&self) -> NumericBucket {
        NumericBucket {
            constants: Vec::new(),
            variables: self.variables.clone(),
            expressions: self.expressions.clone(),
        }
    }

    pub fn intersection(&self, other: &NumericBucket) -> NumericBucket {
        let mut constants = self.constants.clone();
        constants.retain(|c| other.constants.contains(c));
        let mut variables = self.variables.clone();
        variables.retain(|v| other.variables.contains(v));
        let mut expressions = self.expressions.clone();
        expressions.retain(|e| other.expressions.contains(e));
        NumericBucket {
            constants,
            variables,
            expressions,
        }
    }

    pub fn is_all_multiples(&self) -> bool {
        let all_multiple = self.expressions.iter().all(|e| match e {
            NumericExpression::Multiplication(_) => true,
            _ => false,
        });
        self.constants.is_empty() && self.variables.is_empty() && all_multiple
    }
}

impl<'a> Iterator for NumericBucketIter<'a> {
    type Item = NumericExpression;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            0 => {
                if self.index < self.bucket.constants.len() {
                    let c = self.bucket.constants[self.index].clone();
                    self.index += 1;
                    Some(NumericExpression::Constant(c))
                } else {
                    self.state = 1;
                    self.index = 0;
                    self.next()
                }
            }
            1 => {
                if self.index < self.bucket.variables.len() {
                    let v = self.bucket.variables[self.index].clone();
                    self.index += 1;
                    Some(NumericExpression::Variable(v))
                } else {
                    self.state = 2;
                    self.index = 0;
                    self.next()
                }
            }
            2 => {
                if self.index < self.bucket.expressions.len() {
                    let e = self.bucket.expressions[self.index].clone();
                    self.index += 1;
                    Some(e)
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
    }
}

impl<'a> Iterator for NumericBucketIterMut<'a> {
    type Item = NumericExpressionMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                0 => {
                    if let Some(c) = self.constants.next() {
                        return Some(NumericExpressionMut::Constant(c));
                    } else {
                        self.state = 1;
                    }
                }
                1 => {
                    if let Some(v) = self.variables.next() {
                        return Some(NumericExpressionMut::Variable(v));
                    } else {
                        self.state = 2;
                    }
                }
                2 => {
                    if let Some(e) = self.expressions.next() {
                        return Some(NumericExpressionMut::Expression(e));
                    } else {
                        return None;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

impl IntoIterator for NumericBucket {
    type Item = NumericExpression;
    type IntoIter = NumericBucketIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        NumericBucketIntoIter {
            constants: self.constants.into_iter(),
            variables: self.variables.into_iter(),
            expressions: self.expressions.into_iter(),
            state: 0,
        }
    }
}

impl<'a> IntoIterator for &'a mut NumericBucket {
    type Item = NumericExpressionMut<'a>;
    type IntoIter = NumericBucketIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl Iterator for NumericBucketIntoIter {
    type Item = NumericExpression;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                0 => {
                    if let Some(c) = self.constants.next() {
                        return Some(NumericExpression::Constant(c));
                    } else {
                        self.state = 1;
                    }
                }
                1 => {
                    if let Some(v) = self.variables.next() {
                        return Some(NumericExpression::Variable(v));
                    } else {
                        self.state = 2;
                    }
                }
                2 => {
                    return if let Some(e) = self.expressions.next() {
                        Some(e)
                    } else {
                        None
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

impl FromIterator<NumericExpression> for NumericBucket {
    fn from_iter<T: IntoIterator<Item = NumericExpression>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();

        let mut bucket = NumericBucket {
            constants: Vec::with_capacity(lower),
            variables: Vec::with_capacity(lower),
            expressions: Vec::with_capacity(lower),
        };

        for expr in iter {
            bucket.push(expr);
        }

        bucket
    }
}

impl fmt::Display for NumericBucket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        write!(f, "[")?;
        if let Some(first) = iter.next() {
            write!(f, "{:?}", first)?;
            for expr in iter {
                write!(f, ", {:?}", expr)?;
            }
        }
        write!(f, "]")
    }
}
