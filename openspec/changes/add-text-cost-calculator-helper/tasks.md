本 tasks 清单记录通用 text cost calculator helper 的后续探索和实施入口，并把 Markdown adapter 作为首个接入方。审计已将本 change 收敛到“一组统一接口的 text -> cost helper functions”的共享 helper 边界内。

## 1. 阻塞级审计

- [x] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“一组统一接口的 helper functions 接收纯文本并输出 protocol-compatible cost measurement”这一核心目标。
- [x] 1.2 审计本 change 是否基于已归档 `structure-protocol-fields-and-readable-output` 的 current raw protocol `cost.measurements[]` shape，且没有在本 change 中另行固定协议字段结构。
- [x] 1.3 审计 capability ID 是否只复用现有 `docnav-contracts`、`adapter-protocol` 和 `markdown-navigation`，不新增 capability。
- [x] 1.4 审计当前 change 是否只包含 `openspec/changes/add-text-cost-calculator-helper/` 下的 artifacts，且没有修改主规范、schema、example 或实现代码。

## 2. 方案收敛

- [x] 2.1 确认 helper 的 shared crate owner 和 public API：新增 `docnav-text-cost`，暴露 `line_cost(text)`, `byte_cost(text)`, `token_cost(text)`，并返回 current `Measurement` 兼容结果。
- [x] 2.2 确认 helper 首期提供的 `lines`、`bytes` 和 `tiktoken-rs` `o200k_base` token cost functions、每个函数的 unit、scope 附加方式和 readable formatter 边界；measurement 顺序由调用方拥有。
- [x] 2.3 审计 `tiktoken-rs` `o200k_base` token helper function 的初始化方式、crate 版本、许可、离线构建、性能和 release 影响；结论写入 design：使用 `tiktoken-rs` `0.12.0`、MIT、Rust 1.85+、`o200k_base_singleton()`、ordinary plain-text token counting，依赖下载后不依赖运行期网络或外部 tokenizer 文件。
- [x] 2.4 确认 Markdown adapter 首期在哪些 operation/scope/helper function 使用 helper 结果：read result 使用 `selection` scope，outline full entry 与 heading section entry 使用 `entry` scope，并按 `lines`、`bytes`、`tokens` 顺序报告。
- [x] 2.5 确认 Markdown `limit` 首期继续保持 Unicode 字符预算；token cost 不作为 pagination budget。

## 3. 实施与验证

- [x] 3.1 实现 shared text cost calculator helper functions，覆盖纯文本输入、空文本、Unicode、换行、字节数、scope 不由 helper 附加、`tiktoken-rs` `o200k_base_singleton()` 和 ordinary plain-text token counting 边界。
- [x] 3.2 更新 Markdown adapter cost 计算，改为由 Markdown adapter 选择 helper functions，并把已选中文本作为唯一 required input 交给 helper，在 current `cost.measurements[]` 内输出 helper 结果。
- [x] 3.3 同步 owner 主规范、schema/example、fixture 和测试，证明 raw protocol shape 与 readable summary 分层不变。
- [x] 3.4 运行范围匹配的 Rust、schema/example、OpenSpec 和 workspace 验证，并确认 CI/release Rust toolchain 满足 `tiktoken-rs` `0.12.0` 的 Rust 1.85+ floor。
