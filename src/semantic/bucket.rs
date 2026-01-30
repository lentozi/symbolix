use crate::lexer::constant::Number;
use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::variable::Variable;
use std::fmt;
use std::fmt::Formatter;
use std::vec::IntoIter;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct NumericBucket {
    pub constants: Vec<Number>,
    pub variables: Vec<Variable>,
    pub expressions: Vec<NumericExpression>,
}

// 自定义迭代器
// state = 0: 常量
// state = 1: 变量
// state = 2: 表达式
pub struct NumericBucketIter<'a> {
    bucket: &'a NumericBucket,
    state: u8,
    index: usize,
}

pub struct NumericBucketIntoIter {
    constants: IntoIter<Number>,
    variables: IntoIter<Variable>,
    expressions: IntoIter<NumericExpression>,
    state: u8,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct LogicalBucket {
    pub constants: Vec<bool>,
    pub variables: Vec<Variable>,
    pub expressions: Vec<LogicalExpression>,
}

pub struct LogicalBucketIter<'a> {
    bucket: &'a LogicalBucket,
    state: u8,
    index: usize,
}

pub struct LogicalBucketIntoIter {
    constants: IntoIter<bool>,
    variables: IntoIter<Variable>,
    expressions: IntoIter<LogicalExpression>,
    state: u8,
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
        let mut vars: Vec<NumericExpression> = self.variables.clone().into_iter().map(NumericExpression::variable).collect();
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

impl LogicalBucket {
    pub fn new() -> Self {
        LogicalBucket {
            constants: Vec::new(),
            variables: Vec::new(),
            expressions: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.constants.len() + self.variables.len() + self.expressions.len()
    }

    pub fn push(&mut self, expr: LogicalExpression) {
        match expr {
            LogicalExpression::Constant(c) => self.constants.push(c),
            LogicalExpression::Variable(v) => self.variables.push(v),
            _ => self.expressions.push(expr),
        }
    }

    pub fn extend(&mut self, other: LogicalBucket) {
        self.constants.extend(other.constants);
        self.variables.extend(other.variables);
        self.expressions.extend(other.expressions);
    }

    pub fn execute_constant(&mut self, op_and: bool) {
        if op_and {
            let result = self.constants.iter().all(|&c| c);
            self.constants.clear();
            self.constants.push(result);
        } else {
            let result = self.constants.iter().any(|&c| c);
            self.constants.clear();
            self.constants.push(result);
        }
    }

    pub fn iter(&self) -> LogicalBucketIter<'_> {
        LogicalBucketIter {
            bucket: self,
            state: 0,
            index: 0,
        }
    }
}

impl<'a> Iterator for LogicalBucketIter<'a> {
    type Item = LogicalExpression;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            0 => {
                if self.index < self.bucket.constants.len() {
                    let c = self.bucket.constants[self.index];
                    self.index += 1;
                    Some(LogicalExpression::Constant(c))
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
                    Some(LogicalExpression::Variable(v))
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

impl IntoIterator for LogicalBucket {
    type Item = LogicalExpression;
    type IntoIter = LogicalBucketIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        LogicalBucketIntoIter {
            constants: self.constants.into_iter(),
            variables: self.variables.into_iter(),
            expressions: self.expressions.into_iter(),
            state: 0,
        }
    }
}

impl Iterator for LogicalBucketIntoIter {
    type Item = LogicalExpression;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                0 => {
                    if let Some(c) = self.constants.next() {
                        return Some(LogicalExpression::Constant(c));
                    } else {
                        self.state = 1;
                    }
                }
                1 => {
                    if let Some(v) = self.variables.next() {
                        return Some(LogicalExpression::Variable(v));
                    } else {
                        self.state = 2;
                    }
                }
                2 => {
                    return self.expressions.next();
                }
                _ => unreachable!(),
            }
        }
    }
}

impl FromIterator<LogicalExpression> for LogicalBucket {
    fn from_iter<T: IntoIterator<Item = LogicalExpression>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();

        let mut bucket = LogicalBucket {
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

impl fmt::Display for LogicalBucket {
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
