本 change 记录将 Markdown 文档 cost 从单纯文件大小估算改为 token-informed 估算的想法；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前 outline display 和 read cost 常以行数加 KB 表示，KB 更接近文件传输大小，不直接反映 AI 读取大型文档时真正关心的上下文消耗。引入 token-based document cost 可以让 `outline -> ref -> read` 的选择依据更接近模型预算，减少读者把“文件很小”误判为“上下文很便宜”的情况。

## What Changes

- 将 Markdown adapter 生成的 section cost / read cost 从单纯文件字节大小或 KB 展示，改为 token-informed 文档大小估算。
- 记录 Rust `tiktoken` crate 作为首选实现方向，后续实现前需要审计 crate 名称、维护状态、encoding 选择、离线可用性和构建影响。
- 保持 `limit_chars` 的现有分页语义不变：本 change 不把分页预算从字符数改成 token 数。
- 保持协议和 readable 输出的字段 shape 不变：`cost` 和 outline/find `display` 仍是 adapter 生成的可读字符串，不新增机器必需字段。
- 非目标：本 change 不要求所有格式 adapter 立刻统一 token cost，也不引入跨 adapter 的 token budget 参数。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `docnav-contracts`: 澄清 `cost` 是 adapter 生成、core 透传的可读成本摘要，token-informed 展示不得改变协议字段 shape 或 `limit_chars` 语义。
- `markdown-navigation`: 将 Markdown section/read cost 的计算依据从文件大小为主调整为 token-informed 估算，并约束 display/read cost 的可读表达和兼容边界。

## Impact

- 受影响代码：`docnav-markdown` 的 cost 计算、outline display 组装、read result cost 生成，以及相关 fixture/test expectation。
- 受影响文档：`docs/protocol.md`、`docs/adapters/markdown.md`、`docs/examples/` 和对应 OpenSpec specs/examples 中出现 `lines | KB` 或 cost 语义的位置。
- 受影响依赖：可能新增 Rust `tiktoken` crate；实现前需要确认是否使用固定 encoding、是否需要缓存 tokenizer、是否允许离线构建和测试。
- 兼容性：协议 JSON schema 字段类型保持不变；人类可读 `cost` 文案会变化，因此依赖 exact display string 的测试和示例需要同步更新。
