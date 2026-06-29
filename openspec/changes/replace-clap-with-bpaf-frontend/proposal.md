本 change 用 `bpaf` 替换当前 clap-based argv frontend，同时保持 Docnav 的标准参数语义边界。

## Why

Docnav 的大部分 CLI 参数语义已经由标准参数流程承接。继续让 clap command/arg builder 同时承担 argv 解析、help、默认值、枚举、范围和 typed validation，会让 CLI frontend 与标准参数 owner 之间的职责重复。

目标是降低 CLI 前端复杂度，而不是把同一组语义搬到另一个 parser 库。`bpaf` 作为目标依赖，是因为它能提供可复用的 argv parsing 和 help 能力，同时在调研中表现出更好的热度与维护活跃度信号。

## What Changes

- 引入 `bpaf` 作为 direct CLI argv frontend 的实现依赖，替换 clap 在 direct CLI 路径上的解析和 help surface。
- frontend 输出 command/subcommand、positionals、raw flag values、help request 和 frontend diagnostics。
- 参数语义、defaults、required、range、enum、operation applicability、source merge 和 strict validation 继续由标准参数流程或 owning native option handler 负责。
- help 可以复用 `bpaf` 的渲染能力，但数据来源必须是 command context、standard parameter metadata 和 adapter native option metadata。
- 继续保留 unknown argv、extra positional、unused operation flag 的兼容 warning 行为。
- 不在本 change 修改 protocol-json、adapter invoke stdin JSON、readable output 或 config file semantics。

## Success Criteria

- 未被当前 operation 消费的 known flag 不会因为 typed value 不合法而提前失败。
- 被当前 operation 消费的参数仍按标准参数流程严格校验。
- core CLI 和 adapter direct CLI 的 help 仍覆盖 usage、defaults、possible values 和 adapter native options。
- help 不通过读取 config 或执行 adapter operation 来获得参数语义。

## Capabilities

### New Capabilities

- `cli-argv-frontend`: direct CLI argv classification、frontend mapping 和 metadata-driven help 边界。

### Modified Capabilities

当前 change 新增 `cli-argv-frontend` capability。审计门禁会确认 archive 时是否需要并入 `core-cli` 或保留为独立 capability。

## Impact

- 影响 `docnav-cli-args`、core CLI parser、adapter direct CLI args/help 和 CLI help/warning tests。
- 可能移除或降级 `clap` direct CLI 用途，并新增 `bpaf`。
- 不改变 protocol-json、adapter invoke stdin JSON、readable output 或 config file semantics。
