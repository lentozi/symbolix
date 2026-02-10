#[macro_export]
macro_rules! var {
    ($name:expr, $var_type:expr, $init_val:expr) => {
        $crate::semantic::variable::Variable::new($name, $var_type, $init_val)
    };
}

#[macro_export]
macro_rules! numeric_bucket {
    () => {
        $crate::semantic::bucket::NumericBucket::new()
    };

    ( $( $expr:expr ),+ $(,)? ) => {{
        let iter = ::core::iter::IntoIterator::into_iter([
            $( $expr ),+
        ]);
        iter.collect::<$crate::semantic::bucket::NumericBucket>()
    }};

    ( $expr:expr ; $n:expr ) => {{
        let mut bucket = $crate::semantic::bucket::NumericBucket::new();
        for _ in 0..$n {
            bucket.push($expr.clone());
        }
        bucket
    }};
}

#[macro_export]
macro_rules! logical_bucket {
    () => {
        $crate::semantic::bucket::LogicalBucket::new()
    };

    ( $( $expr:expr ),+ $(,)? ) => {{
        let iter = ::core::iter::IntoIterator::into_iter([
            $( $expr ),+
        ]);
        iter.collect::<$crate::semantic::bucket::LogicalBucket>()
    }};

    ( $expr:expr ; $n:expr ) => {{
        let mut bucket = $crate::semantic::bucket::LogicalBucket::new();
        for _ in 0..$n {
            bucket.push($expr.clone());
        }
        bucket
    }};
}
