/// 新建编译上下文，可嵌套使用，嵌套使用时在线程内创建栈结构的编译上下文，在闭包结束后退栈
#[macro_export]
macro_rules! new_compile_context {
    { $($body:tt)* } => {{
        let mut ctx_arc = std::sync::Arc::new($crate::context::CompileContext::new());

        $crate::context::CompileContext::push_current(&ctx_arc, |ctx| {
            $($body)*
        })
    }};
}

/// 在编译上下文中获取上下文对象
#[macro_export]
macro_rules! with_compile_context {
    ($ctx:ident, $($body:tt)*) => {{
        if let Some($ctx) = $crate::context::CompileContext::current() {
            $($body)*
        } else {
            panic!("No current CompileContext found");
        }
    }};
}

/// 在编译上下文中新建变量作用域，可嵌套使用，作用域采用栈结构，在闭包结束后退栈
#[macro_export]
macro_rules! new_var_scope {
    { $(body:tt)* } => {{
        if let Some(ctx) = $crate::context::CompileContext::current() {
            ctx.with_new_scope(|ctx| {
                $($body)*
            })
        } else {
            panic!("No current CompileContext found");
        }
    }}
}

/// 向编译上下文添加编译错误
#[macro_export]
macro_rules! push_compile_error {
    ($($error:expr),+ $(,)?) => {{
        if let Some(ctx) = $crate::context::CompileContext::current() {
            $(ctx.push_error($error);)*
        } else {
            panic!("No current CompileContext found");
        }
    }};
}

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
