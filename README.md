# symbolix

一个用 Rust 实现的表达式解析、语义分析与编译工具集（workspace），包含核心运算模块和用于在编译期生成表达式计算代码的 `proc-macro`模块。适合作为表达式语言解析、优化与嵌入计算功能的基础库或学习示例。

主要 crates:
- `symbolix-core` — 解析（lexer / parser）、语义（semantic）、优化（optimizer）与上下文管理等核心功能。
- `symbolix-compile` — 一个 proc-macro（`compile!`），用于在编译期把表达式编译为可直接计算的代码/结构。

## 特性
- Pratt parser 风格的表达式解析
- 词法分析器（`Lexer`）
- 抽象语法树（AST）到语义树的转换
- 简单表达式优化器（常量折叠等）
- 编译时宏（`compile!`）将字符串表达式嵌入生成的可执行代码
- 示例项目包含可视化（依赖 `tree-drawer`）用于调试和展示树结构

## 需求
- Rust 1.60+（使用 2021 edition 的代码）
- 推荐使用 `cargo` 管理构建与测试
- 若运行带 GUI 的示例，需要能够打开图形窗口（`tree-drawer` 依赖）

## 仓库结构概览
- `Cargo.toml` — workspace 定义（顶层）
- `symbolix-core/` — 核心库
  - `src/` — 源代码目录（`lexer`, `parser`, `semantic`, `optimizer`, `context` 等）
  - `examples/` — 核心示例（包含可视化）
- `symbolix-compile/` — proc-macro crate
  - `src/` — 宏实现
  - `examples/` — 使用 `compile!` 的示例
- `examples/` — 顶层示例（可选）

## 快速开始

1. 克隆并进入项目根目录（包含 workspace `Cargo.toml`）：
```bash
# 在本地（示例命令）：
git clone https://github.com/lentozi/symbolix.git symbolix
cd symbolix
```

2. 构建整个 workspace：
```bash
cargo build --workspace
```

3. 只构建特定 crate（可选）：
```bash
cargo build -p symbolix-core
cargo build -p symbolix-compile
```

## 运行示例

- `symbolix-core` 的示例展示了 Lexer、Parser、语义转换、优化，并使用 `tree-drawer` 打开可视化窗口（需 GUI 环境）：
```rust
use symbolix_core::lexer::symbol::Precedence;
use symbolix_core::lexer::Lexer;
use symbolix_core::optimizer::optimize;
use symbolix_core::parser::expression::Expression;
use symbolix_core::parser::pratt_parsing;
use symbolix_core::semantic::semantic_without_ctx;
use symbolix_core::semantic::variable::VariableType;
use symbolix_core::var;
use tree_drawer::egui_viewer::TreeViewer;
// ... 示例中会解析表达式、优化并展示树结构
```

- `symbolix-compile` 的示例演示如何使用编译期宏 `compile!` 直接在代码中嵌入表达式并计算：
```rust
use symbolix_compile::compile;

fn main() {
    let code = compile!("-x + y + 123 + 45.67 * ((89 - 0.1) ^ x) ^ x + 0");
    println!("{}", code.calculate(1.0, 100.0));

    let code = compile!("x + y");
    println!("{}", code.calculate(1.0, 100.0));
}
```

## 常见用法说明

- 解析与语义转换流程（高层）：
  1. 使用 `Lexer::new(input)` 将字符串转为 token 流。
  2. 使用 Pratt parsing（`pratt_parsing`）构造 AST。
  3. 使用 `semantic_without_ctx` 将 AST 转换为语义树。
  4. 调用 `optimize` 对语义树进行简单优化（例如常量折叠）。
  5. （可选）将生成的语义结构用于运行时或通过 `compile!` 在编译期生成代码。

## API 摘要
- `symbolix_core::lexer::Lexer` — 词法分析器入口类型。
- `symbolix_core::parser::pratt_parsing` — Pratt parser 入口函数，用于解析表达式。
- `symbolix_core::semantic::semantic_without_ctx` — AST -> 语义树转换。
- `symbolix_core::optimizer::optimize` — 语义树优化。
- `symbolix_compile::compile!` — proc-macro，将字符串表达式在编译期编译为可用对象（示例中生成的对象具有 `calculate` 方法）。

## 测试
- 运行 workspace 测试：
```bash
cargo test --workspace
```
- 各 crate 下也可能包含单独的测试与示例，可使用 `cargo test -p <crate>` 或 `cargo run -p <crate> --example <name>` 运行。

## 开发指南
- 代码风格：遵循常见的 Rust 风格（`rustfmt` / `clippy` 推荐）。
- 本项目使用模块化设计：`lexer`、`parser`、`semantic`、`optimizer` 等分别负责不同阶段。
- 如果你要扩展解析规则或增加运算符，请优先在 `symbolix-core/src/parser` 与 `symbolix-core/src/lexer` 中进行修改，并补充相应的测试与示例。
- 进行改动后，请运行 `cargo test --workspace` 并查看示例是否按预期工作。

## 贡献
- 欢迎提交 issue 与 PR。请在 PR 中：
  - 清晰描述变更目的与范围；
  - 添加/更新相应测试与示例；
  - 遵循现有代码风格并运行测试确保没有回归。

## 许可证
- 本项目采用MIT许可。项目根目录下包含完整的许可文本文件：
  - `LICENSE`：MIT 许可证（Copyright (c) 2026 lentozi）
- `symbolix/Cargo.toml` 中的 `license` 字段已设置为 `MIT`，并保留了 `license-file = "LICENSE"` 以兼容常见工具。
- 使用或再分发本项目时，请保留原始的版权声明与许可文件，并遵循所选许可的条款。
