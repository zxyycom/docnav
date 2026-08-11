## Why

Docnav 当前为 document operation 维护 `protocol-json`、`readable-json` 和 `readable-view` 三种 public output shape。`readable-json` 与 `protocol-json` 同时承担 machine-readable contract，增加了 schema、projection 和 consumer 的维护成本。

Document pipeline 已经产出完整 `ProtocolResponse`。本 change 让 protocol serialization 与 readable rendering 直接消费这个统一结构：稳定机器接口保留 `protocol-json`，presentation 由代码注入的 renderer 生成文本。

## Scope

- Shared output paths：`ProtocolJson` 与 `Rendered(RenderStrategy)`。
- Renderer contract：接收一个不可变 `ProtocolResponse`，返回完整 UTF-8 `String` 或 `RenderFailure`。
- Public CLI/config values：`readable-view` 与 `protocol-json`；省略 output 或选择 `readable-view` 时，core CLI 注入内置 renderer。
- Linked code API：直接调用 shared output API 的代码可以注入自定义 renderer；renderer identity 不进入 CLI、config 或 serialized contract。
- Breaking boundary：删除 `readable-json` mode 及其 public schema、examples 和 validation surface；已有值按普通 invalid-value behavior 拒绝。

本 change 只修改 output contract 与 core CLI mapping；protocol、navigation、adapter、diagnostics、logging 和 release owners 保持现有 contract。

## What Changes

1. 将 shared document output model 收敛为 `ProtocolJson` 与 `Rendered(RenderStrategy)`。
2. 让两条路径接收同一个 `ProtocolResponse`：protocol path 直接序列化，rendered path 把 response 交给选定 renderer。
3. 将现有 readable rendering 包装为内置 `readable-view` renderer，并由 core composition 为默认/显式 `readable-view` 注入。
4. 删除 `readable-json` output branch 和 public contract，同步更新 docs、schema、examples、fixtures 与 tests。

## Done When

1. Document CLI/config 只接受 `readable-view` 与 `protocol-json`。
2. `ProtocolJson` 序列化既有 `ProtocolResponse`；`Rendered` 将同一个结构原样交给选定 renderer。
3. Renderer 成功时，stdout 等于 renderer 返回的完整文本；renderer 失败时不写 stdout，也不切换 renderer。
4. Built-in renderer 保持现有 `readable-view` 最终文本和 framing contract。
5. Runtime、Current docs 和 validation materials 不再发布 `readable-json` output contract，且 protocol contract 保持不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `output-contract`：定义两条 output paths、统一 `ProtocolResponse` 输入、renderer injection 和文本写入边界。
- `core-cli`：将 document output values 收敛为 `readable-view` 与 `protocol-json`，并注入内置 renderer。

## Impact

- 影响 `docnav-output`、`docnav-readable`、core CLI output composition 和相关 tests。
- 影响 `docs/architecture.md`、`docs/output.md`、`docs/cli.md`、`docs/navigation.md` 以及 readable schema/example/fixture/conformance materials。
- 已保存 `defaults.output: readable-json` 的 config 和显式 `--output readable-json` 调用需要迁移到 `protocol-json`、`readable-view` 或默认输出。
