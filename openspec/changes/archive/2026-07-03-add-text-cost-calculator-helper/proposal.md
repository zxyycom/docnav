本 change 记录为文档 cost 引入通用 text cost calculator helper 的方向。当前实现尚未开始；本 change 不重新定义已经归档并进入主规范的 raw protocol `cost.measurements[]` shape。

## Why

当前 Markdown adapter 已通过 raw protocol `cost.measurements[]` 报告 `lines` 和 `bytes`，readable 输出再从这些 measurement 派生成本摘要。这个计算本质上不依赖 Markdown：任何格式最终只要产生纯文本输出，就可以对该文本计算 cost。本 change 把这类机械计算收敛成通用 helper，让调用方选择需要的 helper function，并把已选中的纯文本作为唯一输入，即可获得 protocol-compatible `Measurement`。Markdown adapter 是首个消费者，基于 `tiktoken-rs` `o200k_base` 的 token cost 是首期必交付目标。

## What Changes

- 为 shared library 增加通用 text cost calculator helper，由一组同签名函数组成；每个函数只接收纯文本并返回对应 protocol-compatible `Measurement`。
- helper 使用 current `Measurement` 语义输出 `unit + value`，不附加 `scope`；函数自身定义 unit，调用方不传 unit 参数或策略对象。
- 首期 helper function 集合由新 shared crate `docnav-text-cost` 暴露：`line_cost(text: &str) -> Measurement` -> `lines`、`byte_cost(text: &str) -> Measurement` -> `bytes`、`token_cost(text: &str) -> Measurement` -> `tokens`。
- `token_cost` 使用 `tiktoken-rs` `0.12.0` 的 `o200k_base` tokenizer，按 ordinary plain-text tokenization 计算，调用方不能选择 tokenizer dependency、encoding 或 model preset。
- Markdown adapter 自己选择要调用哪些 helper function、scope 和排序，并直接调用 helper 计算其已经选中的输出文本 cost；首期替换现有 Markdown read selection 与 outline entry cost 计算，并按 `lines`、`bytes`、`tokens` 顺序报告。
- 保持 shared helper 只负责“text -> cost measurement”，不负责选择 helper function 集合、选择文档区域、解析格式、决定分页预算，也不决定 readable 输出的成本摘要格式。
- 以 current protocol/readable owner 为边界：raw protocol 继续使用 `cost.measurements[]`，readable 输出继续由 output 层从 measurements 派生成本摘要。
- 非目标：本 change 不开放用户选择分页预算单位，不把 token cost 自动用作 pagination budget，不要求 helper 自动生成默认 measurement 集合，不要求所有调用方必须在 public output 中展示某种 cost，也不开放调用方选择 tokenizer dependency、encoding 或 model preset。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `docnav-contracts`: 为跨组件共享的 text cost calculator helper 留出边界。
- `adapter-protocol`: 确认 helper 输出继续映射到 current `cost.measurements[]` protocol shape。
- `markdown-navigation`: 使用 shared helper 计算 Markdown adapter 已选中文本的 text cost。

## Impact

- 受影响代码：新增 `docnav-text-cost` crate、`docnav-markdown` cost 计算、readable cost summary 兼容性检查、相关测试和 fixture。
- 受影响文档：共享 helper 边界、Markdown adapter cost 说明、protocol/readable owner 引用，以及必要的 schema/example 证明材料。
- 依赖关系：`structure-protocol-fields-and-readable-output` 已归档，current raw protocol cost shape 是 `cost.measurements[]`；本 change 只在该 shape 内增加可复用 text cost calculator 能力。`tiktoken-rs` `0.12.0` 为 MIT license，要求 Rust 1.85+；本 change 接受该 Rust floor，并要求实现验证 CI/release toolchain 满足该要求。
