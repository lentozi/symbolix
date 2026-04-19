# exprion

`exprion` 是一个用 Rust 编写的表达式工具链 workspace，围绕“表达式字符串 -> 词法分析 -> 语法分析 -> 语义 IR -> 优化 -> 运行时编译 / 代码生成”这条链路组织。当前仓库包含表达式前端 `exprion-core`、面向使用者的组合 API crate `exprion-api`、运行时 JIT crate `exprion-engine`，以及已有的编译期宏 crate `exprion-compile`。

它适合用于：

- 构建自定义表达式语言
- 在 Rust 项目中嵌入公式计算能力
- 研究 Pratt parser、语义分析和简单代数化简
- 在运行时把表达式 JIT 编译为可执行代码
- 在 Rust 项目中保留编译期宏展开这条现有能力

## 当前能力

- 词法分析：支持整数、浮点、科学计数法、布尔字面量和 Unicode 标识符
- Pratt parser：支持括号、一元前缀运算、二元运算、关系运算和三元条件表达式
- 语义分析：将 AST 转换为数值 / 逻辑语义 IR
- 优化：包含常量折叠、扁平化、规范化等优化步骤
- 方程能力：支持单变量一次方程求解
- 运行时 JIT：
  - 当前提供 `exprion-engine::jit_compile_numeric(...)` 和 `exprion-engine::jit_compile_logical(...)`
  - 当前通过 LLVM C API 生成并执行本机代码
  - 当前已支持基础算术、`pow`、关系逻辑和数值 `piecewise` 的机器码生成
- 编译期代码生成：
  - `formula!`：从表达式字符串生成可调用对象
  - `exprion!`：从块状 DSL 生成可调用对象

## Workspace 结构

- `exprion-core/`
  - 表达式核心库
  - 包含 `lexer`、`parser`、`semantic`、`optimizer`、`equation`、`context` 等模块
- `exprion-api/`
  - 面向使用者的组合 API
  - 统一表达式、变量、常量之间的组合方式
- `exprion-compile/`
  - `proc-macro` crate
  - 提供 `formula!` 和 `exprion!`
- `exprion-engine/`
  - 运行时 JIT crate
  - 当前将 `exprion-core` 产出的数值语义 IR 降到 LLVM IR，再交给 LLVM MCJIT 执行
  - 内部已拆分为 `lowering` 和 `backend` 两层，`backend` 目前通过 trait 接 MCJIT，方便后续切换到 ORC JIT
- `examples/`
  - 顶层示例，演示如何通过 `exprion` facade 使用公开能力
- `src/lib.rs`
  - 顶层 facade crate
  - 默认导出 `exprion-api` 的高层能力、编译期宏，以及少量 JIT 能力
  - 底层 crate 通过 `exprion::advanced::{core, engine}` 暴露给高级使用者
- `documents/`
  - 项目文档或设计资料

## 支持的表达式语法

- 数值与布尔字面量：`1`、`3.14`、`1.23e-4`、`true`、`false`
- 标识符：`x`、`_tmp`、`变量`、`αβγ`
- 一元运算：`+x`、`-x`、`!flag`
- 二元运算：`+`、`-`、`*`、`/`、`%`、`^`、`&&`、`||`
- 关系运算：`<`、`>`、`<=`、`>=`、`==`、`!=`
- 三元表达式：`cond ? a : b`
- 分组：`(x + y) * z`

## 环境要求

- Rust stable（2021 edition）
- `cargo`

## 快速开始

```bash
cargo build --workspace
```

运行测试：

```bash
cargo test --workspace
```

运行示例：

```bash
cargo run -p exprion-core --example pipeline
cargo run -p exprion-core --example equation
cargo run -p exprion-core --example error

cargo run -p exprion-compile --example formula
cargo run -p exprion-compile --example rust_analyse
cargo run -p exprion-engine --example basic

cargo test -p exprion-api
cargo test -p exprion-engine

cargo run --example workspace_demo
cargo run --example facade
```

## JIT 方向

当前推荐把 `exprion-core` 理解为前端和优化层，把 `exprion-engine` 理解为运行时后端：

- `exprion-core` 负责 lexer、parser、语义分析和优化
- `exprion-engine` 负责把优化后的语义 IR 降到 LLVM IR 并执行
- `exprion-compile` 继续保留为编译期宏能力，但它不是 JIT

面向普通使用者时，当前更推荐直接从顶层 `exprion` crate 开始；需要底层能力时，再进入 `exprion::advanced`。

顶层 facade 的一个最小 JIT 用法：

```rust
use exprion::{
    advanced::core::{
        lexer::Lexer,
        new_compile_context,
        optimizer::optimize,
        parser::Parser,
        semantic::Analyzer,
    },
    compile_numeric,
};

fn main() {
    let semantic = new_compile_context! {
        let mut lexer = Lexer::new("z + x * 2 + 1");
        let parsed = Parser::pratt(&mut lexer);
        let mut analyzer = Analyzer::new();
        let mut semantic = analyzer.analyze_with_ctx(&parsed);
        optimize(&mut semantic);
        semantic
    };
    let compiled = compile_numeric(semantic.into()).unwrap();

    let result = compiled.calculate(&[3.0, 10.0]).unwrap();
    assert_eq!(compiled.variables(), vec!["x".to_string(), "z".to_string()]);
    assert!((result - 17.0).abs() < 1e-9);
}
```

若你正在构建底层流水线，也可以直接使用 `exprion-core` 和 `exprion-engine`：

```rust
use exprion_core::{
    lexer::Lexer,
    new_compile_context,
    optimizer::optimize,
    parser::Parser,
    semantic::Analyzer,
};
use exprion_engine::jit_compile_numeric;

fn main() {
    let semantic = new_compile_context! {
        let mut lexer = Lexer::new("z + x * 2 + 1");
        let parsed = Parser::pratt(&mut lexer);
        let mut analyzer = Analyzer::new();
        let mut semantic = analyzer.analyze_with_ctx(&parsed);
        optimize(&mut semantic);
        semantic
    };
    let compiled = jit_compile_numeric(semantic).unwrap();

    // 变量顺序按名字排序，这里是 (x, z)
    let result = compiled.calculate(&[3.0, 10.0]).unwrap();
    assert_eq!(compiled.variables(), vec!["x".to_string(), "z".to_string()]);
    assert!((result - 17.0).abs() < 1e-9);
}
```

其中：

- `jit_compile_numeric(...)` 和 `jit_compile_logical(...)` 负责把 `SemanticExpression` 编译为运行时可调用对象
- 字符串到 `SemanticExpression` 的前端转换由 `exprion-core` 负责，JIT 不再直接接收 `&str`

当前限制：

- 仅支持数值表达式的 JIT
- 仅支持 `f64` 调用接口
- 依赖本机可用的 LLVM 安装和 `llvm-config`
- 当前不支持布尔变量参数的 JIT；逻辑表达式目前支持基于数值关系的逻辑组合

## 使用 `exprion`

顶层 `exprion` 是推荐的默认入口，适合大多数库使用者。

```rust
use exprion::{scope, Var};

fn main() {
    scope(|| {
        let x = Var::number("x");
        let y = Var::number("y");

        let expr = (x.clone() + y.clone() * 2.0).gt(10.0);
        println!("{}", expr.semantic());
    });
}
```

其中：

- `Expr`、`Var`、`Equation`、`scope` 等高层能力来自 facade
- `formula!` 和 `exprion!` 也可以直接从 `exprion` 导入
- 只有在你明确需要 lexer、parser、semantic 这些底层组件时，才建议进入 `exprion::advanced`

## 使用 `exprion::advanced::core`

当你需要手动控制词法分析、语法分析、语义分析和优化流程时，可以走底层入口。

下面的示例展示了从字符串表达式到语义 IR 的基本流程：

```rust
use exprion_core::{
    lexer::Lexer,
    new_compile_context,
    optimizer::optimize,
    parser::Parser,
    semantic::Analyzer,
};

fn main() {
    new_compile_context! {
        let mut lexer = Lexer::new("x > 100 ? x * (2 + 3) : x / 2");
        let expression = Parser::pratt(&mut lexer);

        let mut analyzer = Analyzer::new();
        let mut semantic = analyzer.analyze_with_ctx(&expression);
        optimize(&mut semantic);

        println!("{}", semantic);
    }
}
```

说明：

- `new_compile_context!` 会创建编译上下文，并在分析过程中管理变量注册与错误收集
- 未显式声明类型的变量会根据上下文推断为数值变量或布尔变量
- 数值变量当前默认推断为 `f64`

## 求解单变量一次方程

`exprion::advanced::core` 当前提供单变量一次方程求解能力：

```rust
use exprion_core::{
    equation::Equation,
    lexer::Lexer,
    new_compile_context,
    optimizer::optimize,
    parser::Parser,
    semantic::Analyzer,
};

fn main() {
    new_compile_context! {
        let mut lexer = Lexer::new("2 * x + 3 == 9");
        let expression = Parser::pratt(&mut lexer);

        let mut semantic = Analyzer::new().analyze_with_ctx(&expression);
        optimize(&mut semantic);

        let equation = Equation::new(semantic);
        let result = solve!(equation).unwrap();
        println!("x = {}", result);
    }
}
```

当前限制：

- 只支持等式形式的表达式
- 只支持单变量一次方程
- 幂、多变量方程和更复杂的分段方程暂未支持

## 使用 `formula!`

`formula!` 会在编译期解析和优化表达式，并返回一个匿名对象。该对象当前至少包含：

- `calculate(...)`
- `to_closure()`

示例：

```rust
use exprion::formula;

fn main() {
    let formula = formula!("y + x * 2");

    // 参数顺序按变量名字母序排列，这里是 (x, y)
    println!("{}", formula.calculate(3.0, 10.0));

    let gt = formula!("x > 100");
    println!("{}", gt.calculate(120.0));
}
```

补充说明：

- `calculate(...)` 的参数顺序按变量名排序，而不是按表达式中首次出现的顺序
- 数值表达式返回 `f64`
- 逻辑表达式返回 `bool`

## 使用 `exprion!`

`exprion!` 提供了一个更接近 Rust 代码风格的块状 DSL，适合把多个中间表达式组合起来并在最后返回一个表达式或元组。

示例：

```rust
use exprion::exprion;

fn main() {
    let code = exprion! {
        let x = var!("x", f64);
        let z = var!("z", f64);

        let expr = if x >= 10.0 {
            expr!("z + 20")
        } else {
            expr!("z * 2")
        };

        (expr + x, x)
    };

    // 参数顺序按真实变量名排序，即 (x, z)
    println!("{}", code.calculate(10.0, 5.0).0);
}
```

这个 DSL 当前支持的核心元素包括：

- `var!("name", ty)` 声明变量
- `expr!("...")` 从字符串表达式生成语义表达式
- 普通算术 / 逻辑 / 关系表达式
- `if / else`
- 以表达式或元组作为块的最终返回值

## 已有示例文件

- `exprion-core/examples/pipeline.rs`
  - 演示手动执行 lexer、parser、semantic、optimizer 流程
- `exprion-core/examples/equation.rs`
  - 演示方程求解
- `exprion-core/examples/error.rs`
  - 演示错误输入的处理方式
- `exprion-compile/examples/formula.rs`
  - 演示 `formula!`
- `exprion-compile/examples/rust_analyse.rs`
  - 演示 `exprion!`
- `examples/facade.rs`
  - 演示通过顶层 `exprion` facade 使用变量、关系、方程和 JIT
- `examples/workspace_demo.rs`
  - 顶层入口示例

## 开发建议

- 若要扩展表达式语法，优先查看 `exprion-core/src/lexer` 和 `exprion-core/src/parser`
- 若要扩展语义能力或优化规则，优先查看 `exprion-core/src/semantic` 和 `exprion-core/src/optimizer`
- 若要扩展 JIT 后端，优先查看 `exprion-engine/src/lib.rs`、`exprion-engine/src/lowering.rs` 和 `exprion-engine/src/backend/mcjit.rs`
- 若要扩展编译期宏行为，重点查看 `exprion-compile/src/lib.rs`、`exprion-compile/src/codegen.rs` 和 `exprion-compile/src/rust_expr.rs`
- 修改后建议至少运行一次 `cargo test --workspace`

## 许可证

本项目采用 MIT 许可证，详见根目录中的 `LICENSE` 文件。



