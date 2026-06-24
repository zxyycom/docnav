本 change 只起草用 lexopt 替换 clap 作为 argv frontend 的想法和审计入口；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Why

Docnav 的 CLI 参数语义应由标准参数流程拥有，CLI frontend 只负责 argv tokenization 和入口映射。继续让 derive-style CLI parser 承担语义会妨碍 adapter runtime native options 和 loose warning 策略。

## What Changes

- 记录 `lexopt` 是目标 argv frontend。
- CLI frontend 只做 tokenization/frontend mapping，不拥有参数语义、默认值、operation applicability 或最终校验。
- help 输出继续展示 usage、defaults、possible values 和 adapter native options，但来源应是标准参数 metadata 与 owner metadata。
- 保留 unknown argv、extra positional、unused operation flag warning。
- 不在本 change 设计标准参数核心或 consumer 迁移细节。

## Capabilities

### New Capabilities

- `cli-argv-frontend`: direct CLI argv tokenization、frontend mapping 和 help generation 边界。

### Modified Capabilities

当前草案不直接修改已归档主 spec；审计门禁会确认是否需要改为 `core-cli` delta。

## Impact

- 未来会影响 `docnav-cli-args`、core CLI、adapter direct CLI 和 help tests。
- 可能移除或降级 `clap` 依赖，并新增 `lexopt`。
- 不改变 protocol-json、adapter invoke stdin JSON、readable output 或 config file semantics。
