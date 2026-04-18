use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::variable::Variable;
use std::fmt;
use std::fmt::Formatter;
use std::vec::IntoIter;

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

    pub fn extend(&mut self, other: &LogicalBucket) {
        self.constants.extend(other.constants.clone());
        self.variables.extend(other.variables.clone());
        self.expressions.extend(other.expressions.clone());
    }

    pub fn append(&mut self, other: &mut LogicalBucket) {
        self.constants.append(&mut other.constants);
        self.variables.append(&mut other.variables);
        self.expressions.append(&mut other.expressions);
    }

    pub fn execute_constant(&mut self, op_and: bool) {
        if self.constants.len() <= 1 {
            return;
        }

        let result = if op_and {
            self.constants.iter().all(|&c| c)
        } else {
            self.constants.iter().any(|&c| c)
        };
        self.constants.clear();
        self.constants.push(result);
    }

    pub fn iter(&self) -> LogicalBucketIter<'_> {
        LogicalBucketIter {
            bucket: self,
            state: 0,
            index: 0,
        }
    }

    pub fn remove_true(&mut self) {
        self.constants.retain(|&c| !c);
    }

    pub fn remove_false(&mut self) {
        self.constants.retain(|&c| c);
    }

    pub fn single_item(&self) -> Option<LogicalExpression> {
        if self.len() != 1 {
            return None;
        }
        if let Some(constant) = self.constants.first() {
            return Some(LogicalExpression::Constant(*constant));
        }
        if let Some(variable) = self.variables.first() {
            return Some(LogicalExpression::Variable(variable.clone()));
        }
        self.expressions.first().cloned()
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
