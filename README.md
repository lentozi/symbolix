# symbolix

`symbolix` 是一个用 Rust 编写的表达式工具链 workspace，围绕“表达式字符串 -> 词法分析 -> 语法分析 -> 语义 IR -> 优化 -> 求值 / 代码生成”这条链路组织。当前仓库包含核心分析库 `symbolix-core` 和编译期宏 crate `symbolix-compile`。

它适合用于：

- 构建自定义表达式语言
- 在 Rust 项目中嵌入公式计算能力
- 研究 Pratt parser、语义分析和简单代数化简
- 在编译期把表达式编译为可直接调用的 Rust 代码

## 当前能力

- 词法分析：支持整数、浮点、科学计数法、布尔字面量和 Unicode 标识符
- Pratt parser：支持括号、一元前缀运算、二元运算、关系运算和三元条件表达式
- 语义分析：将 AST 转换为数值 / 逻辑语义 IR
- 优化：包含常量折叠、扁平化、规范化等优化步骤
- 方程能力：支持单变量一次方程求解
- 编译期代码生成：
  - `compile!`：从表达式字符串生成可调用对象
  - `symbolix_rust!`：从块状 DSL 生成可调用对象

## Workspace 结构

- `symbolix-core/`
  - 表达式核心库
  - 包含 `lexer`、`parser`、`semantic`、`optimizer`、`equation`、`context` 等模块
- `symbolix-compile/`
  - `proc-macro` crate
  - 提供 `compile!` 和 `symbolix_rust!`
- `examples/`
  - 顶层示例，演示如何直接使用编译期宏
- `src/lib.rs`
  - 顶层 crate 目前仅作为 workspace 入口，没有额外公开 API
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
- 首次构建时需要拉取 Git 依赖 `tree-drawer`

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
cargo run -p symbolix-core --example compile
cargo run -p symbolix-core --example equation
cargo run -p symbolix-core --example error

cargo run -p symbolix-compile --example main
cargo run -p symbolix-compile --example rust_analyse

cargo run --example workspace_demo
```

## 使用 `symbolix-core`

下面的示例展示了从字符串表达式到语义 IR 的基本流程：

```rust
use symbolix_core::{
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

`symbolix-core` 当前提供单变量一次方程求解能力：

```rust
use symbolix_core::{
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
        let result = equation.solve().unwrap();
        println!("x = {}", result);
    }
}
```

当前限制：

- 只支持等式形式的表达式
- 只支持单变量一次方程
- 幂、多变量方程和更复杂的分段方程暂未支持

## 使用 `compile!`

`compile!` 会在编译期解析和优化表达式，并返回一个匿名对象。该对象当前至少包含：

- `calculate(...)`
- `to_closure()`

示例：

```rust
use symbolix_compile::compile;

fn main() {
    let formula = compile!("y + x * 2");

    // 参数顺序按变量名字母序排列，这里是 (x, y)
    println!("{}", formula.calculate(3.0, 10.0));

    let gt = compile!("x > 100");
    println!("{}", gt.calculate(120.0));
}
```

补充说明：

- `calculate(...)` 的参数顺序按变量名排序，而不是按表达式中首次出现的顺序
- 数值表达式返回 `f64`
- 逻辑表达式返回 `bool`

## 使用 `symbolix_rust!`

`symbolix_rust!` 提供了一个更接近 Rust 代码风格的块状 DSL，适合把多个中间表达式组合起来并在最后返回一个表达式或元组。

示例：

```rust
use symbolix_compile::symbolix_rust;

fn main() {
    let code = symbolix_rust! {
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

- `symbolix-core/examples/compile.rs`
  - 演示手动执行 lexer、parser、semantic、optimizer 流程
- `symbolix-core/examples/equation.rs`
  - 演示方程求解
- `symbolix-core/examples/error.rs`
  - 演示错误输入的处理方式
- `symbolix-compile/examples/main.rs`
  - 演示 `compile!`
- `symbolix-compile/examples/rust_analyse.rs`
  - 演示 `symbolix_rust!`
- `examples/workspace_demo.rs`
  - 顶层入口示例

## 开发建议

- 若要扩展表达式语法，优先查看 `symbolix-core/src/lexer` 和 `symbolix-core/src/parser`
- 若要扩展语义能力或优化规则，优先查看 `symbolix-core/src/semantic` 和 `symbolix-core/src/optimizer`
- 若要扩展编译期宏行为，重点查看 `symbolix-compile/src/lib.rs`、`symbolix-compile/src/codegen.rs` 和 `symbolix-compile/src/rust_expr.rs`
- 修改后建议至少运行一次 `cargo test --workspace`

## 许可证

本项目采用 MIT 许可证，详见根目录中的 `LICENSE` 文件。
