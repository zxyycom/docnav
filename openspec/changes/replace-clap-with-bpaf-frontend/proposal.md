本 change 原计划用 `bpaf` 替换当前 clap-based argv frontend。经 `adopt-strict-input-boundaries` Track A 协调后，active artifacts 改为保留 `clap` 作为 strict core CLI parser/help owner，同时继续收缩 frontend 与标准参数语义边界。

## Why

Docnav 的大部分 CLI 参数语义已经由标准参数流程承接。strict core CLI contract 要求 parser/help surface 稳定、未知输入和 operation-inapplicable 输入在入口边界失败，并把参数语义留给标准参数 owner。

目标是降低 CLI 前端复杂度，而不是把同一组语义搬到另一个 parser 库。当前 active direction 保留 `clap` 的 parsing/help 能力，并把 `docnav-cli-args` 收缩为 strict parser/mapper 需要的共享能力。

## What Changes

- 保留 `clap` 作为 core CLI strict parser/help owner；本 change 不再引入 `bpaf` 替换 active core CLI parser/help surface。
- frontend 输出 command/subcommand、positionals、raw flag values、help request 和 frontend diagnostics。
- 参数语义、defaults、required、range、enum、operation applicability、source merge 和 strict validation 继续由标准参数流程或 owning native option handler 负责。
- help 使用 `clap` surface，但数据来源必须是 command context、standard parameter metadata 和 adapter native option metadata。
- unknown argv、extra positional 和 operation-inapplicable flag 在 strict core CLI contract 下投影为 primary input diagnostic，而不是进入 adapter execution。
- 不在本 change 修改 protocol-json、protocol request handling、readable output 或 config file semantics。

## Success Criteria

- unknown argv、extra positional 和 operation-inapplicable flag 在入口边界投影为 primary input diagnostic。
- 被当前 operation 消费的参数仍按标准参数流程严格校验。
- core CLI help 仍覆盖 usage、defaults、possible values 和 adapter native options。
- help 不通过读取 config 或执行 adapter operation 来获得参数语义。

## Capabilities

### New Capabilities

- `cli-argv-frontend`: core CLI argv classification、frontend mapping 和 metadata-driven help 边界。

### Modified Capabilities

当前 change 新增 `cli-argv-frontend` capability。审计门禁会确认 archive 时是否需要并入 `core-cli` 或保留为独立 capability。

## Impact

- 影响 `docnav-cli-args`、core CLI parser、CLI help/diagnostic tests。
- 不移除或降级 `clap` core CLI parser/help 用途；若未来重新评估其它 parser 库，必须作为独立协调 change 处理。
- 不改变 protocol-json、protocol request handling、readable output 或 config file semantics。
