本 design 将“obvious result”定义为一个直接可验证的事实：当前 structured base result 中返回的非空 opaque ref 去重后恰好只有一个。Auto-read 默认、静默生效；只有既有 `read` 成功时才增加可观察结果。

## Context

Docnav 的基础阅读链路是 `outline -> ref -> read`，搜索链路常见形式是 `find -> ref -> read`。当当前返回结果只包含一个 distinct ref 时，调用方没有候选决策需要完成；core 可以在保持 adapter 单次 operation 语义的前提下直接追加一次 read。

当前实现已经让 document success/failure 在 output plan 前形成统一 `ProtocolResponse`，并由 `ProtocolJson` 或注入 renderer 的 `Rendered` 消费。本 change 在 `docnav-navigation` 内完成 base operation 与 read 的组合，再把一个不可变 response 交给 output layer。

## Goals / Non-Goals

**Goals:**

- 为 `outline` 和 `find` 提供默认启用、可通过 CLI 或 project/user config 控制的 unique-ref auto-read。
- 仅按当前返回 items 的 distinct opaque ref 判断是否直接 read。
- 复用当前 selected adapter 和既有 read strategy，不重新解析 ref 或递归调用 CLI。
- 只在追加 read 成功时向 typed protocol result 增加原因和完整 `ReadResult`。
- 未触发或追加 read 未成功时，保持 base protocol/readable result 和退出行为不变。
- 让 `protocol-json`、`readable-view` 和 invocation logging 消费同一 composed outcome。

**Non-Goals:**

- 不新增 adapter operation、adapter input field、protocol request argument、error code 或 auto-read 专用参数。
- 不用 request page、response continuation 或全量结果完整性定义“唯一”；只判断当前返回结果。
- 不公开 skipped reason、failed status 或 nested read diagnostic。
- 不使用 rank、label、query 相似度、模型判断或 ref grammar 推断。
- 不引入 generic continuation envelope、批量 read、outline preview 或第二套 read 语义。
- 不改变 adapter-owned outline/find/read facts、分页算法或 ref 解析语义。

## Decisions

### Decision 1: 一个 canonical field 同时拥有 CLI 与 config surface

Core parameter catalog 声明一个 string enum field：

- Canonical identity：`docnav.defaults.auto_read`。
- 合法值：`disabled`、`unique-ref`。
- CLI locator：`--auto-read`。
- Config locator：`defaults.auto_read`。
- Built-in default：`unique-ref`。
- Merge strategy：`Replace`。
- Operation bindings：`outline`、`find`。
- Core consumer：navigation orchestration projection。
- Environment locator：无。

Project 和 user config 使用同一 JSON shape：

```json
{
  "defaults": {
    "auto_read": "disabled"
  }
}
```

`outline` 和 `find` 按 `explicit CLI > project config > user config > built_in` 解析最终 mode。省略所有来源时使用 `unique-ref`；CLI 可以覆盖 project/user config，project config 可以覆盖 user config，user config 可以覆盖 built-in default。

`read`、`info` 和 non-document command 不接受 `--auto-read`。有效的 `defaults.auto_read` 是已知 config shape，但只在 outline/find selected-operation view 中参与解析；其它 operation 不执行 auto-read。非法 CLI value 或非法 config enum 在 adapter dispatch 前进入现有 source-attributed input diagnostic。

`docnav config inspect` 通过 canonical field metadata 识别 `defaults.auto_read`，报告 project/user source scope、locator、字段 identity 和 value，不构造 document operation 或触发 auto-read。

### Decision 2: unique-ref 只表示当前返回 ref 唯一

Base operation 成功并通过 protocol 校验后，navigation：

1. 对 structured outline 的 `entries[].ref` 或 find 的 `matches[].ref` 做 string-exact 去重。
2. 只有 distinct 集合恰好包含一个非空 opaque ref 时执行一次 read。
3. 集合为空或包含多个值时，直接返回 base response。

Core 不解析或改写 ref。多个 find matches 指向同一 ref 时，distinct 集合仍为一个，因此只 read 一次。

Request page 和 response `page` 不参与判定。即使当前请求不是第一页或 base response 仍有 continuation，只要当前返回 ref 唯一，仍执行 read；该判断不声称整个文档或全部搜索结果只有一个 ref。Unstructured outline 没有返回 ref，因此保持原 response。

### Decision 3: nested read 是既有 read 的内部编排

追加 read 使用同一个 normalized document path、同一个 selected adapter definition 和原样 candidate ref，并从 read page `1` 开始。`limit`、pagination normalization 和其它 read input 继续遵守现有 read contract；本 change 不定义新的 auto-read 参数或资源控制。

Nested read 复用内部 typed dispatch seam；不递归进入 top-level CLI，不启动子进程，不再次选择 adapter，也不执行第二个 output plan。只有 selected adapter read strategy 解析 ref。

### Decision 4: owner 按现有调用链分层

- `docnav` core parameter catalog 和 CLI parser/help 拥有 canonical field、CLI/config locators、default、source priority、strict applicability 和 core projection。
- `docnav-navigation` 在 base response 校验成功后执行 distinct ref 判定、复用 selected adapter 调用 read，并选择 base 或 composed result。
- Format adapter 继续只执行单次 outline/find/read strategy，不接收 auto-read mode。
- `docnav-protocol` 拥有 serialized optional `auto_read` success object。
- `docnav-output` 与 `docnav-readable` 拥有 composed result 到 readable header/block 的映射。

### Decision 5: `auto_read` 只表达成功追加

只有 mode 为 `unique-ref`、当前返回 ref 唯一、nested read 成功且 composed response 校验通过时，outline/find result 才包含：

```json
{
  "auto_read": {
    "reason": "unique_ref",
    "read": {
      "ref": "<opaque-ref>",
      "content": "<content>",
      "content_type": "<media-type>",
      "cost": {"measurements": []},
      "page": null
    }
  }
}
```

`reason` 说明追加 read 由当前返回 ref 唯一触发；`read` 直接复用完整 existing `ReadResult`。Object 使用封闭字段集合，不增加 `mode`、`status`、sibling `ref` 或 nested error。

现有 base fields 保持同层、同含义和原有顺序承诺。Outer response `operation` 仍为 `outline` 或 `find`；nested read 不创建第二个 public envelope。

### Decision 6: 未产生成功追加时静默返回 base response

以下情况都返回已校验的 base response，不增加 `auto_read`：

- Auto-read mode 经 CLI/config/default resolution 得到 `disabled`。
- Base result 没有返回 ref，或 distinct ref 不等于一个。
- Nested read 返回 protocol diagnostic 或其它非成功 outcome。
- 无法形成有效的 composed success result。

Base operation failure 继续使用现有 `ProtocolResponse::Failure` 和退出码。Base success 后未产生成功追加时，CLI 保持 base success 和退出码 `0`；public protocol 和 readable output 不解释未追加原因。

### Decision 7: 两条 output path 消费同一 response

`ProtocolJson` 直接序列化 navigation 选择的 base 或 composed `ProtocolResponse`。内置 `readable-view` renderer 保留 base outline/find header；仅当 `auto_read` 存在时：

- Header 映射 `reason` 和 nested read 的 ref、content type、cost summary、page。
- Nested read content 使用 block pointer `/auto_read/read/content`。
- Renderer 发出一个 length-delimited block，payload 等于 `ReadResult.content`。

`auto_read` 不存在时，两条 output path 都使用现有 base projection，不增加 header field 或 block。Output layer 不执行 read，也不维护 renderer-only selection 或 failure facts。

### Decision 8: default behavior、compatibility 与 invocation logging

该 change 不修改 protocol version。`auto_read` 是 additive optional success field，但因为 built-in default 为 `unique-ref`，符合条件的默认 invocation 会执行一次额外 read，并在成功时出现该字段。需要稳定保留单 operation 行为的调用方可以在 CLI、project config 或 user config 中选择 `disabled`。

Invocation log 的 root operation 仍为 `outline` 或 `find`，不新增独立 nested-read top-level event。Logging 可以记录 bounded auto-read attempt outcome，但不得把 skipped reason、nested diagnostic 或正文加入 public result。显式启用 content capture 时，成功追加的 read content 使用既有 hash/capture 机制和既有 event shape；正文不进入主 JSONL event。

## Risks / Trade-offs

- **默认行为增加一次内部 dispatch。** 只在当前返回 ref 唯一时发生，调用方可通过 CLI 或 config 选择 `disabled`。
- **同一 mode 来自多个输入来源。** Canonical field 使用 `Replace` 和既有 `explicit > project > user > built_in` 顺序，config inspection 与 provenance 验证最终来源。
- **“唯一”只针对当前返回结果。** `reason: "unique_ref"` 的 contract 明确定义为当前 result 的 distinct ref，不代表全部分页结果全局唯一。
- **Nested read 未成功时没有 public 原因。** 这是静默编排的有意行为；启用 invocation logging 时可保留 bounded 诊断事实。
- **成功追加会扩大 response。** Nested `ReadResult` 继续使用既有 read limit/page 语义和 continuation 字段。
- **Optional field 默认可能出现。** Schema、examples、consumer fixtures 和 renderer conformance 必须同步接受该 additive field；需要稳定旧 shape 的 caller 可显式禁用。

## Migration Plan

1. 先更新 CLI、navigation、protocol、output、architecture 和 testing owner 文档，定稿 CLI/config locators、source priority、当前返回 ref 判定和 success-only mapping。
2. 同步 config/protocol schema、examples、config inspection fixtures、renderer conformance 和 invocation content-capture fixtures。
3. 增加 catalog/source resolution/core projection 和 strict CLI/config tests，再实现 navigation composition 与 optional success result。
4. 接入 readable renderer、invocation logging 和 end-to-end tests。
5. 运行全工作区验证并完成局部 contract/compatibility 审计。

## Open Questions

无未回答问题。实现前用户批准门已完成，其执行状态与完成记录以 `tasks.md` 0.1 为准。
