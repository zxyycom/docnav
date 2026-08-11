本 proposal 为 `outline` 和 `find` 增加默认生效的 unique-ref auto-read：当当前返回结果中的非空 ref 去重后恰好只有一个时，Docnav 在同一次 invocation 中直接复用既有 `read`，减少一次机械调用。

## Why

`outline -> ref -> read` 和 `find -> ref -> read` 是 Docnav 的稳定基础链路。当一次 outline 或 find 返回的 ref 只有一个 distinct value 时，调用方已经没有候选选择需要完成，下一次 `read` 只是机械往返。

本 change 将这一步作为默认、静默的 core orchestration。判定只比较当前返回结果中的 opaque ref 字符串，不使用 rank、文本相似度、模型判断、分页完整性推断或 ref grammar。

## What Changes

- Core parameter catalog 为 `outline` 和 `find` 声明同一个 auto-read field：CLI locator 为 `--auto-read`，project/user config locator 为 `defaults.auto_read`，合法值为 `disabled|unique-ref`，built-in default 为 `unique-ref`。
- Auto-read 使用现有 `explicit CLI > project config > user config > built_in` 替换优先级；第一版不声明 environment locator。调用方既可按 invocation 覆盖，也可在项目或用户配置中持久设置。
- Base operation 成功并通过校验后，core 对当前 structured result 的 `entries[].ref` 或 `matches[].ref` 去重。非空 distinct ref 集合恰好为一个时，直接执行一次既有 `read`。
- 判定只针对当前返回结果，不要求 request page 为 `1`，也不要求 response `page` 为 null。多个 find matches 指向同一 ref 时仍触发一次 read。
- 追加 read 复用同一 normalized document path、selected adapter、opaque ref 和既有 read 输入语义；不新增 operation、adapter input、protocol request argument、分页模型或 auto-read 专用参数。
- 只有追加 read 成功且 composed result 校验通过时，base result 才增加 `auto_read: { "reason": "unique_ref", "read": <ReadResult> }`。未触发、追加 read 未成功或显式禁用时，返回原 base response，不增加状态、原因或错误分支。
- `protocol-json` 直接序列化 composed `ProtocolResponse`；内置 renderer 从同一 response 生成 `readable-view`，并把成功追加的 read content 放入 `/auto_read/read/content` block。
- Base operation failure、退出码和 output failure 继续使用现有边界。Auto-read 不创建第二个 public envelope 或第二个 top-level invocation event。

## Capabilities

### New Capabilities

- 无。该 change 修改既有 core CLI 编排和 document result/output contract，不创建新的长期 capability。

### Modified Capabilities

- `core-cli`: 定义 default-on unique-ref auto-read 的 CLI/config surface、config inspection、退出行为和 invocation logging 边界。
- `navigation-input-resolution`: 定义 auto-read field 的 canonical locators、source priority、operation binding、core projection、当前返回 ref 判定和既有 read dispatch orchestration。
- `protocol-contract`: 定义 outline/find result 中仅成功时出现的 optional `auto_read` object。
- `output-contract`: 定义 `ProtocolJson` 与内置 `readable-view` renderer 对同一 composed response 的投影和 block framing。

## Impact

- 影响 core parameter catalog、document CLI parser/help、config source validation/inspection、`docnav-navigation` dispatch orchestration、`docnav-protocol` result model、`docnav-output` mapping、`docnav-readable` renderer config 和 invocation content capture。
- 需要同步更新 `docs/architecture.md`、`docs/cli.md`、`docs/navigation-input-resolution.md`、`docs/protocol.md`、`docs/output.md`、`docs/testing.md`、config/protocol schema 和 examples、renderer conformance vectors 及相关 Rust/smoke tests。
- 默认 invocation 在当前返回 ref 唯一且追加 read 成功时会多一次 adapter read，并增加 optional success field；CLI 或 config 解析为 `disabled` 时保留原单 operation 行为。
- 不新增 adapter operation、adapter input、protocol request argument、error code、ref grammar、auto-read failure status 或 skip reason；普通 `OutlineResult`、`FindResult`、`ReadResult` 的 adapter-owned facts 保持不变。
