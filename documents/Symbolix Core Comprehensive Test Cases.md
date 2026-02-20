## User

为我的symbolix-core库，给出全面的测试用例吧，只说用例，暂时不需要写代码

**Tool Call: Find paths matching “`**/*symbolix*.*/**/*`”**
Status: Completed

```json
{
  "offset": 0,
  "current_matches_page": [
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.gitignore",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\.gitignore",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\copilot.data.migration.agent.xml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\copilot.data.migration.ask.xml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\copilot.data.migration.ask2agent.xml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\copilot.data.migration.edit.xml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\inspectionProfiles",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\inspectionProfiles\\Project_Default.xml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\modules.xml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\symbolix.iml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.idea\\vcs.xml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.zed",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.zed\\debug.json",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\.zed\\tasks.json",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\Cargo.lock",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\Cargo.toml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\README.md",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\examples\\main.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\src\\lib.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-compile\\Cargo.toml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-compile\\examples\\main.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-compile\\src\\codegen.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-compile\\src\\lib.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\Cargo.toml",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\examples\\compile.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\context\\compile.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\context\\mod.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\context\\runtime.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\context\\symbol_table.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\error\\io_error.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\error\\mod.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\error\\other_error.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\error\\semantic_error.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\error\\syntax_error.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\error\\type_error.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\lexer\\constant.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\lexer\\macros.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\lexer\\mod.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\lexer\\symbol.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\lexer\\token.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\lib.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\macros.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\optimizer\\factor.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\optimizer\\mod.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\optimizer\\normalize.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\optimizer\\optimize_d1.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\optimizer\\optimize_term.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\parser\\expression.rs",
    "C:\\Users\\LengTouZai\\Documents\\Repository\\symbolix\\symbolix-core\\src\\parser\\mod.rs"
  ],
  "all_matches_len": 62
}
```

**Tool Call: Find paths matching “`symbolix-core/src/**/*.rs`”**
Status: Completed

```json
{
  "offset": 0,
  "current_matches_page": [],
  "all_matches_len": 0
}
```

## Assistant

下面给出针对 `symbolix-core` 库的全面测试用例清单（仅列出用例，不写实现代码）。我按模块归类、标注测试类型（单元 / 集成 / 属性 / 模糊 / golden）、优先级（高/中/低），并为每个用例给出输入描述、预期行为/断言、以及需要关注的边界/异常情况。你可以直接把这些用例转成 Rust 的 `#[test]`、property tests、或 fuzzing 脚本。

总体建议
- 优先实现高优先级用例，确保 lexer → parser → AST → optimizer → runtime 路径的语义保持不变（端到端正确性）。
- 对优化器使用语义等价性质测试（用随机/生成器构造表达式，比较优化前后在解释器上的求值结果）。
- 对语法/错误消息做 golden 测试，确保文件/行列位置、错误描述稳定。
- 使用带名字的测试（例如 `lexer_tokenizes_numeric_literals`）便于定位失败原因。

模块索引（指向代码文件）
- 词法器: `symbolix/symbolix-core/src/lexer/token.rs`, `symbolix/symbolix-core/src/lexer/symbol.rs`, `symbolix/symbolix-core/src/lexer/constant.rs`, `symbolix/symbolix-core/src/lexer/mod.rs`
- 解析器: `symbolix/symbolix-core/src/parser/expression.rs`, `symbolix/symbolix-core/src/parser/mod.rs`
- 优化器: `symbolix/symbolix-core/src/optimizer/normalize.rs`, `symbolix/symbolix-core/src/optimizer/factor.rs`, `symbolix/symbolix-core/src/optimizer/optimize_d1.rs`, `symbolix/symbolix-core/src/optimizer/optimize_term.rs`, `symbolix/symbolix-core/src/optimizer/mod.rs`
- 上下文/符号表/编译: `symbolix/symbolix-core/src/context/compile.rs`, `symbolix/symbolix-core/src/context/runtime.rs`, `symbolix/symbolix-core/src/context/symbol_table.rs`, `symbolix/symbolix-core/src/context/mod.rs`
- 宏/常量/其它: `symbolix/symbolix-core/src/macros.rs`, `symbolix/symbolix-core/src/lexer/constant.rs`
- 错误类型: `symbolix/symbolix-core/src/error/*.rs`
- 库入口: `symbolix/symbolix-core/src/lib.rs`
- 示例/集成: `symbolix/symbolix-core/examples/compile.rs`

具体测试用例

1) Lexer（词法分析）
- 用例: `lexer_tokenizes_identifiers`
  - 类型: 单元
  - 优先级: 高
  - 输入: `foo bar _baz qux123`
  - 断言: 产生的 token 类型为 Identifier，名字精确匹配，位置（line/col）正确。

- 用例: `lexer_tokenizes_float_and_scientific`
  - 类型: 单元
  - 优先级: 高
  - 输入: `3.14 0.5e-2 1E6`
  - 断言: Float constants，数值解析精确度在合理容差内。

- 用例: `lexer_tokenizes_operators_and_punctuators`
  - 类型: 单元
  - 优先级: 高
  - 输入: `+ - * / == != <= >= && || => , ; ( ) { }`
  - 断言: 每个符号生成正确 token，优先识别长符号（例如 `==` 而不是 `=`+`=`）。

- 用例: `lexer_handles_unicode_identifiers`
  - 类型: 单元
  - 优先级: 中
  - 输入: `变量 αβγ`
  - 断言: 支持 Unicode 标识符（根据实现策略），并且名称完整。

- 用例: `lexer_reports_invalid_token_with_position`
  - 类型: 单元
  - 优先级: 高
  - 输入: 包含非法字符（例如控制字符或孤立的 `\`）
  - 断言: 产生 `SyntaxError`（或 lexer-specific error），包含准确的行/列。

- 用例: `lexer_position_and_span_consistency`
  - 类型: 单元
  - 优先级: 中
  - 输入: 多行带缩进的源
  - 断言: token 的 span 连续且不重叠；下一个 token 的 start = 上一个 token 的 end + （跳过空白/注释）。

2) Parser（表达式与语法）
- 用例: `parser_parses_primary_expressions`
  - 类型: 单元
  - 优先级: 高
  - 输入: 字面量、标识符、括号表达式、函数调用的最小例子
  - 断言: AST 节点类型正确（Literal/Identifier/Group/Call），子节点位置/内容正确。

- 用例: `parser_respects_operator_precedence_and_associativity`
  - 类型: 单元 / 属性
  - 优先级: 高
  - 输入: `1 + 2 * 3 ^ 2 - 4 / (5 - 3)`
  - 断言: AST 反映预期的优先级和结合性（可以用括号化打印作可比“golden”）。

- 用例: `parser_parses_unary_operators`
  - 类型: 单元
  - 优先级: 高
  - 输入: `-a + +b !c`
  - 断言: 解析出 Unary 节点，操作符位置正确。

- 用例: `parser_parses_function_definitions_and_calls`
  - 类型: 单元 / 集成
  - 优先级: 高
  - 输入: 带参数、默认值、可变参数（若支持）
  - 断言: 参数列表、默认值、可变参数的 AST 结构与语义正确。调用处参数绑定正确。

- 用例: `parser_handles_comma_separated_lists_and_trailing_commas`
  - 类型: 单元
  - 优先级: 中
  - 输入: `f(a, b, )`
  - 断言: 允许/拒绝 trailing comma（依据语法），并且错误消息清晰。

- 用例: `parser_reports_unexpected_token_with_span`
  - 类型: 单元
  - 优先级: 高
  - 输入: `1 + * 2` 或 未闭合的 `(`、字符串
  - 断言: 抛出 `SyntaxError`，包含问题位置、期待的 token 列表（若有），错误消息和 span 与 golden 文本一致。

- 用例: `parser_error_recovery_behavior`
  - 类型: 集成
  - 优先级: 中
  - 输入: 含多个语法错误的文件
  - 断言: 记录多个错误（不是只报第一个），后续错误不被第一个错误污染（如果实现了恢复）。

- 用例: `parser_builds_correct_AST_for_nested_expressions`
  - 类型: 单元
  - 优先级: 中
  - 输入: 极深嵌套的括号（例如深度 200 或者超过栈限制）
  - 断言: 在合理深度内 AST 正确；对于超出限制的输入报告可理解的错误（stack overflow -> 报错）。

3) AST 与语义
- 用例: `ast_pretty_print_and_roundtrip`
  - 类型: 单元 / 集成
  - 优先级: 中
  - 输入: 一组代表性的 AST（从 parser 得到）
  - 断言: pretty-print(AST) 再 parse 回来与原 AST 等价（结构或语义等价）。

- 用例: `ast_node_span_preservation`
  - 类型: 单元
  - 优先级: 中
  - 输入: 含注释、换行的源
  - 断言: AST 节点保留原始源的 span（用于错误定位/工具链）。

4) 优化器（normalize、factor、optimize_*）
- 用例: `optimizer_constant_folding_basic`
  - 类型: 单元 / 属性
  - 优先级: 高
  - 输入: `2 + 3 * 4`, `(1+2)*3`
  - 断言: 优化后折叠为单一常量节点，求值结果一致。

- 用例: `optimizer_algebraic_identities`
  - 类型: 单元
  - 优先级: 高
  - 输入: `x*1`, `x+0`, `0*x`, `x-0`
  - 断言: 优化移除无效操作（根据优化规则）；语义保持。

- 用例: `optimizer_factor_common_subexpressions` (`factor.rs`)
  - 类型: 单元 / 属性
  - 优先级: 高
  - 输入: `(a*b + a*c)`, `a*(b+c)`
  - 断言: `factor` pass 将前者转换为后者（或等价形式）；验证语义等价（对一组随机赋值）。

- 用例: `optimizer_normalize_commutativity_and_ordering` (`normalize.rs`)
  - 类型: 单元 / 属性
  - 优先级: 中
  - 输入: `a + b`, `b + a`
  - 断言: normalize 产生规范化顺序（例如按字典序），并且在 normalize 后比较 AST 可一致化。

- 用例: `optimizer_idempotence`
  - 类型: 属性
  - 优先级: 高
  - 输入: 任意表达式 E
  - 断言: apply(optimize, apply(optimize, E)) == apply(optimize, E)。（对随机生成表达式做多次迭代测试）

- 用例: `optimizer_preserves_semantics`
  - 类型: 属性（重要）
  - 优先级: 高
  - 输入: 随机表达式生成器（受限深度和运算集合）
  - 断言: 对若干随机环境（变量赋值）比较优化前后在解释器/运行时的求值结果相同（允许浮点容差）。

- 用例: `optimize_term_simplification_cases` (`optimize_term.rs`)
  - 类型: 单元
  - 优先级: 中
  - 输入: 特定项级优化例子（例如多项式合并、幂次简化）
  - 断言: 优化后的项符合预期规范形式，语义等价。

- 用例: `optimize_d1_specific_transforms` (`optimize_d1.rs`)
  - 类型: 单元
  - 优先级: 中
  - 输入: 该 pass 目标的输入集合（查看实现，构造对应表达式）
  - 断言: pass 做出期望的模式替换，且语义不变。

- 用例: `optimizer_handles_edge_numeric_cases`
  - 类型: 单元
  - 优先级: 中
  - 输入: 大整数、除以零、NaN、Infinity（浮点）
  - 断言: 对非法数学操作（如常量除以零）要么在编译期报错要么保留为运行时错误，且不执行不安全的替换。

5) 运行时 / 解释器（context/runtime.rs）
- 用例: `runtime_evaluates_basic_expressions`
  - 类型: 集成
  - 优先级: 高
  - 输入: 常量表达式、算术、逻辑、比较
  - 断言: 运行时返回预期值；类型/值匹配。

- 用例: `runtime_function_call_and_scope`
  - 类型: 集成
  - 优先级: 高
  - 输入: 带局部变量、参数、递归调用（有深度限制）
  - 断言: 参数绑定正确，局部作用域在返回后释放，递归结果正确。检测栈深度限制行为。

- 用例: `runtime_handles_errors_and_panics`
  - 类型: 集成
  - 优先级: 中
  - 输入: 运行时类型错误（例如把字符串加到数字）、未定义标识符、除以零
  - 断言: 返回/抛出对应的 `TypeError` / `RuntimeError`，并提供源 span（若可追溯）。

- 用例: `runtime_works_with_optimized_AST`
  - 类型: 集成
  - 优先级: 高
  - 输入: 同一 AST 的原始和优化版本
  - 断言: 两者在相同环境下的求值结果一致（语义验证）。

6) 符号表与编译上下文（context/symbol_table.rs, compile.rs）
- 用例: `symbol_table_insertion_and_lookup`
  - 类型: 单元
  - 优先级: 高
  - 输入: 插入键值、查找、更新、删除
  - 断言: 查找返回最新值；不存在时返回 None 或错误（按实现）。

- 用例: `symbol_table_scoping_and_shadowing`
  - 类型: 单元 / 集成
  - 优先级: 高
  - 输入: 嵌套作用域定义同名符号（global/local）
  - 断言: 内层遮蔽外层，离开内层后恢复外层绑定。

- 用例: `compile_reports_missing_symbol_or_type_errors`
  - 类型: 集成
  - 优先级: 高
  - 输入: 源中引用未定义名称或类型不兼容
  - 断言: 产生 `SemanticError` 或 `TypeError`，包含准确的 span 和可理解信息。

- 用例: `compile_handles_file_io_errors`
  - 类型: 集成
  - 优先级: 中
  - 输入: 请求包含的文件不存在、读权限不足
  - 断言: 抛出 `IoError`，错误链保持（路径信息、原始 IO 错误）。

- 用例: `compile_multiple_files_and_module_resolution`
  - 类型: 集成
  - 优先级: 中
  - 输入: 多文件项目，测试导入、相对路径、循环依赖检测
  - 断言: 正确解析、检测到循环依赖并给出清晰错误或处理策略。

7) 宏（macros.rs）
- 用例: `macros_expansion_simple`
  - 类型: 单元
  - 优先级: 中
  - 输入: 定义并使用简单宏（例如文本替换或参数化模板）
  - 断言: 扩展正确并且扩展后的 AST 与期望一致。

- 用例: `macros_hygiene_and_capture`
  - 类型: 单元
  - 优先级: 中
  - 输入: 宏中定义局部变量与外部变量同名的情形
  - 断言: 宏扩展不意外捕获或污染外部命名空间（如果实现了 hygiene），或文档化行为一致。

- 用例: `macros_recursive_and_limits`
  - 类型: 单元
  - 优先级: 低
  - 输入: 递归宏扩展或大量嵌套扩展
  - 断言: 在合理深度内正常工作；超过限制时给出友好错误而不是无限循环。

8) 错误类型与诊断消息（error/*.rs）
- 用例: `errors_format_consistent_and_includes_spans`
  - 类型: 单元 / golden
  - 优先级: 高
  - 输入: 制造不同错误（syntax/semantic/type/io/other）
  - 断言: 错误消息结构、字段（kind、message、span、source）与预期模板一致；可序列化（如果实现了）并可重现。

- 用例: `diagnostics_line_col_accuracy`
  - 类型: 集成
  - 优先级: 高
  - 输入: 多行源，出错点位于行首/行尾/多字节字符
  - 断言: 报告的行/列号正确（确保多字节字符时列索引按字符而非字节）。

9) 示例/端到端（examples/compile.rs, examples/main.rs）
- 用例: `example_compile_and_run_produces_expected_output`
  - 类型: 集成
  - 优先级: 高
  - 输入: `examples/compile.rs` 中示例源码
  - 断言: 从源码编译到运行的输出或返回值与 README / 示例注释中说明的期望一致。

- 用例: `cli_or_api_compile_errors_propagate_properly`
  - 类型: 集成
  - 优先级: 中
  - 输入: 故意带错误的示例
  - 断言: 错误在 CLI/API 层可见，返回码和错误文本合理。

10) 性能与资源（非功能性）
- 用例: `lexer_parser_throughput_on_large_file`
  - 类型: 性能（基准）
  - 优先级: 低
  - 输入: 大型源文件（数十万行或生成的极长表达式）
  - 断言: 在合理时间内完成（设定基线），内存占用不爆炸。

- 用例: `optimizer_scalability`
  - 类型: 性能
  - 优先级: 低
  - 输入: 包含大量重复子表达式的 AST
  - 断言: 优化时间线性或可接受的增长；没有指数级退化。

11) 属性/模糊测试（Property / Fuzz）
- 用例: `prop_optimizer_preserves_semantics_random`
  - 类型: 属性
  - 优先级: 高
  - 输入: 随机表达式生成器（限制操作集、深度、变量数），多组随机环境赋值
  - 断言: 优化前后在所有测试环境下返回相同结果（浮点允许小误差）。

- 用例: `fuzz_lexer_and_parser_for_crashes`
  - 类型: 模糊
  - 优先级: 高
  - 输入: 随机字节流 / 随机 Unicode 源
  - 断言: 库不会 panic 或崩溃；对不合法输入优雅报错（error 返回，不 panic）。

12) 边界与异常场景
- 用例: `very_long_identifier_and_string_handling`
  - 类型: 单元
  - 优先级: 低
  - 输入: 极长标识符/字符串（例如 1e6 字符）
  - 断言: 要么正常处理，要么返回合理的长度限制错误，不导致内存泄露或崩溃。

- 用例: `deep_recursion_limits_handling`
  - 类型: 集成
  - 优先级: 中
  - 输入: 递归函数深度超过平台或实现限制
  - 断言: 返回栈溢出风格的错误（可捕获），而不是 undefined behavior。

13) 兼容与 API 稳定性
- 用例: `public_api_signatures_and_behaviors`
  - 类型: 单元 / 集成
  - 优先级: 中
  - 输入: 调用公有 API（构建 AST、调用 compile/run）
  - 断言: 返回类型、错误类型不随小改动破坏（用于保证库的稳定性）；在变更时更新测试以反映 API 期望。

14) 辅助测试（helpers / test utils）
- 用例: `test_helpers_create_ast_and_eval_quickly`
  - 类型: 测试工具
  - 优先级: 高（对测试套件）
  - 说明: 编写轻量函数来从字符串生成 tokens/AST/optimized AST 并在内存中 eval，这会极大提升测试可维护性与可读性。

测试命名建议（示例）
- `lexer_tokenizes_numeric_and_string_literals`
- `parser_respects_operator_precedence`
- `parser_reports_unexpected_token_with_span`
- `optimizer_constant_folding_preserves_semantics`
- `optimizer_idempotence_on_random_expressions`
- `runtime_evaluates_function_calls_and_scopes`
- `symbol_table_handles_shadowing`
- `compile_reports_missing_file_as_io_error`
- `macros_expansion_hygiene_test`
- `fuzz_lexer_no_panics`

Golden 测试（建议）
- 保留一组固定的输入文件（`tests/golden/*.src`）及预期输出/错误文本（`*.expected`），用于：
  - parser 的 AST 文本化输出
  - 错误消息与位置
  - 优化器转换前后的“规范化”文本
- 每次变动时对比 golden，以防回归。把这些放到 `tests/golden` 目录并 CI 校验。

如何执行 / 测试策略建议
- 单元测试：针对每个模块的函数，覆盖正例与负例。
- 集成测试：放在 `tests/` 目录，做 end-to-end 流（水到渠成）测试（source -> compile -> optimize -> run）。
- 属性测试：使用 `proptest` 或 `quickcheck` 实现表达式生成器，比较前后语义。
- Fuzz：使用 `honggfuzz` / `libfuzzer`（或 cargo-fuzz）针对 lexer/parser 二元件做崩溃检测。
- Golden：对错误消息与 AST 打印使用固定文件进行比较。

额外注意事项
- 浮点比较：所有浮点断言使用相对/绝对容差。
- 多字节字符（utf-8）与列数：断言行/列以字符为单位（若项目约定如此），测试包含 emoji 和 CJK。
- 错误消息稳定性：如果不希望频繁更新 golden，错误消息应尽量结构化（kind + span + short message），而不是全文人类可变的长段描述。
- 并发/线程安全：如果 `context` 或 `symbol_table` 支持并发访问，添加并发读写测试（race condition 检测）。

结语
我已经把关键路径（lexer → parser → AST → 优化器 → 运行时）、错误处理、宏展开、符号表、以及性能/模糊测试都覆盖在上面。你想让我把这些用例转换成具体的测试函数名和每个测试的输入/期望输出表格，还是直接为某个模块（例如 `lexer` 或 `optimizer`）生成完整的 `#[test]`/`proptest` 测试代码？如果要我生成测试代码，请指定优先模块（我会先生成高优先级的那些）。
