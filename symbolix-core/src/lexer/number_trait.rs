pub trait NumberTrait {
    fn is_integer(&self) -> bool;
    fn is_float(&self) -> bool;
    fn to_integer(&self) -> i64;
    fn to_float(&self) -> f64;
}

impl NumberTrait for f64 {
    fn is_integer(&self) -> bool {
        *self == self.floor()
    }

    fn is_float(&self) -> bool {
        !self.is_integer()
    }

    fn to_integer(&self) -> i64 {
        self.floor() as i64
    }

    fn to_float(&self) -> f64 {
        *self
    }
}

impl NumberTrait for f32 {
    fn is_integer(&self) -> bool {
        *self == self.floor()
    }

    fn is_float(&self) -> bool {
        !self.is_integer()
    }

    fn to_integer(&self) -> i64 {
        self.floor() as i64
    }

    fn to_float(&self) -> f64 {
        *self as f64
    }
}

impl NumberTrait for i64 {
    fn is_integer(&self) -> bool {
        true
    }

    fn is_float(&self) -> bool {
        false
    }

    fn to_integer(&self) -> i64 {
        *self
    }

    fn to_float(&self) -> f64 {
        *self as f64
    }
}

impl NumberTrait for i32 {
    fn is_integer(&self) -> bool {
        true
    }

    fn is_float(&self) -> bool {
        false
    }

    fn to_integer(&self) -> i64 {
        *self as i64
    }

    fn to_float(&self) -> f64 {
        *self as f64
    }
}

impl NumberTrait for u64 {
    fn is_integer(&self) -> bool {
        true
    }

    fn is_float(&self) -> bool {
        false
    }

    fn to_integer(&self) -> i64 {
        *self as i64
    }

    fn to_float(&self) -> f64 {
        *self as f64
    }
}

impl NumberTrait for u32 {
    fn is_integer(&self) -> bool {
        true
    }

    fn is_float(&self) -> bool {
        false
    }

    fn to_integer(&self) -> i64 {
        *self as i64
    }

    fn to_float(&self) -> f64 {
        *self as f64
    }
}
