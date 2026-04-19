# SymPy 对比结果（实测版）

## 说明

本文档基于本机实际运行结果整理而成，目标是比较 `exprion` 和 SymPy 在若干**对齐场景**下的表现。

与之前那份“方案型”报告不同，这一版包含了真实测量结果。

但需要强调：

- 这些结果来自当前这台本机环境
- benchmark 仍然存在一定噪声
- 某些场景是严格可比的，某些场景只能作为近似参考

## 测试环境

### `exprion`

- 仓库：当前本地 workspace
- benchmark 工具：`perf-harness`
- 输入文件：
  [perf_cases_sympy_compare.toml](/c:/Users/DELL/Documents/Repository/symbolix/perf-harness/input/perf_cases_sympy_compare.toml)
- 本轮结果文件：
  [perf_cases_sympy_compare_1776582367.toml](/c:/Users/DELL/Documents/Repository/symbolix/perf-harness/output/perf_cases_sympy_compare_1776582367.toml)

### SymPy

- Python 可执行文件：
  `C:\Users\DELL\.conda\envs\sympy\python.exe`
- SymPy 版本：`1.14.0`

## 对比场景

本次比较分成三类场景：

1. 构建阶段（build）
   意义：从表达式源文本到内部可分析对象的成本。

2. 编译 / 可调用对象生成阶段（compile）
   意义：
   - `exprion`：从语义表达式到 JIT 可执行对象
   - SymPy：从表达式对象到 `lambdify(...)` 生成的 Python callable

3. 重复执行阶段（execute）
   意义：表达式准备完成后，反复输入数值求值的成本。

## 对比表达式

### 1. 默认算术表达式

```text
(((x + 1.25) ^ 2 + y * 3.5 - z / 7.0) * (x - y + 2.0)) / 3.0
```

### 2. 幂运算密集表达式

```text
((x + 1) ^ 2 + (y + 2) ^ 3 + (z + 3) ^ 4) / ((x + 4) ^ -1 + 1)
```

### 3. Piecewise 表达式

`exprion`：

```text
x > 0 ? x * 2 : -x
```

SymPy：

```python
Piecewise((x * 2, x > 0), (-x, True))
```

## 测试方法

### 术语说明

为了避免歧义，这里的几个词含义如下：

- `build`
  指“表达式前端构建成本”，不是 `cargo build` 这种工程编译。
  在当前对比里，它表示把表达式源文本转换成内部表达式对象所需的时间。

- `compile`
  指“把内部表达式进一步变成可执行对象的成本”。
  在 `exprion` 里，这通常对应 JIT 编译；
  在 SymPy 里，这通常对应 `lambdify(...)` 生成可调用对象。

- `execute`
  指“表达式已经准备完成之后，反复输入数值参数执行求值的成本”。
  这是最接近稳态热路径的阶段。

### 通用设置

- build iterations: `5000`
- compile iterations: `500`
- execute iterations: `200000`
- warmup iterations: `200`
- repeat: `7`
- minimum sample duration: `300 ms`

### SymPy 测试方式

- build：`parse_expr(..., evaluate=False)`
- compile：`lambdify((x, y, z), expr, "math")`
- execute：重复调用已构造好的 callable

### `exprion` 测试方式

- build：`build_only`
- compile：`compile_only`
- execute：`execute_only`

## 结果汇总

以下表格优先展示 **mean**，因为这是当前 `perf-harness` 和 SymPy 脚本最容易对齐的值。

### 默认表达式

#### Build

- `exprion` mean: `11376.0 ns`
- SymPy mean: `1048876.7 ns`
- 比值：SymPy 约慢 `92x`

#### Compile

- `exprion` mean: `1911.8 ns`
- SymPy mean: `2259070.9 ns`
- 比值：SymPy 约慢 `1182x`

注意：  
这里 `exprion` 的 compile 场景已经吃到了 compile cache，因此更准确地说，这个结果反映的是“同一表达式反复编译时的缓存命中吞吐”，而不完全等价于“冷启动编译时间”。

#### Execute

- `exprion` mean: `3.5 ns`
- SymPy mean: `537.3 ns`
- 比值：SymPy 约慢 `153x`

### 幂运算密集表达式

#### Build

- `exprion` mean: `13884.8 ns`
- SymPy mean: `860076.9 ns`
- 比值：SymPy 约慢 `62x`

#### Compile

- `exprion` mean: `3877.3 ns`
- SymPy mean: `1465453.1 ns`
- 比值：SymPy 约慢 `378x`

同样说明：  
这里的 `exprion compile` 已经受到 compile cache 的影响，更接近“重复编译缓存命中吞吐”。

#### Execute

- `exprion` mean: `4.5 ns`
- SymPy mean: `813.7 ns`
- 比值：SymPy 约慢 `181x`

这个场景非常值得注意，因为它正好能放大 `exprion-engine` 在整数幂、倒数、平方根专门 lowering 上的收益。

### Piecewise 表达式

#### Execute

- `exprion` mean: `2.1 ns`
- SymPy mean: `735.8 ns`
- 比值：SymPy 约慢 `350x`

但这里也要特别保守：

- SymPy 这条 case 的抖动很高
- `exprion` 这条 case 的执行路径非常短
- 所以这个结果更适合作为“趋势性信号”，不适合作为对外宣传时的主结论

## 结果解读

### 可以明确看到的结论

1. 在当前这组 benchmark 上，`exprion` 在数值执行场景中明显快于 SymPy。

2. 对于重复执行型表达式，尤其是：
   - 幂运算密集
   - piecewise / 分支较轻

   `exprion-engine` 的优势非常明显。

3. 在前端构建上，`exprion-core` 的 parse/build 也明显快于 SymPy 的 `parse_expr`。

### 需要谨慎解读的结论

1. `compile` 对比不能简单理解成“JIT 编译一定比 SymPy 快这么多”，因为当前 `exprion` compile benchmark 会命中 compile cache。

2. `execute` 虽然差距很大，但因为当前机器还有噪声，所以更稳妥的表达是：

   - `exprion` 在这些场景下显著领先 SymPy`
   - 而不是直接对外说“固定快 150 倍、180 倍、350 倍”

3. 这些结果不能推广成“`exprion` 整体强于 SymPy”。

当前结果只支持一个更窄、更准确的结论：

> 在当前测试的数值表达式重复求值场景中，`exprion` 明显快于 SymPy。

## 建议的对外表述

如果你要把这份结果写进 README 或介绍文档，我建议用这种表述：

> 在本地实测的重复数值求值 benchmark 中，`exprion-engine` 在若干代表性表达式上显著快于 SymPy。  
> 对幂运算密集表达式和固定表达式的高频重复执行场景，优势尤其明显。

不要直接写：

- “全面快于 SymPy”
- “整体性能超越 SymPy”
- “通用数学能力优于 SymPy”

## 当前比较的局限

1. 本机结果依赖当前机器环境。
2. benchmark 仍然存在一定抖动。
3. compile 场景里 `exprion` 已经吃到了 cache，严格来说更像“cached compile throughput”。
4. 当前场景主要覆盖的是数值表达式，而不是 SymPy 的完整符号能力。

## 总结

在当前已完成的实测中：

- `exprion` build 明显快于 SymPy
- `exprion` execute 明显快于 SymPy
- 幂运算密集型和固定重复执行型场景，是 `exprion` 当前最有优势的对比点

如果你后面要继续做对外材料，这份报告已经足够作为“技术结论版”的底稿。
