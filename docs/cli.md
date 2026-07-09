# CLI

本文定义 `docnav` CLI 的命令面、命令解析、项目根与路径处理、配置命令、内置 adapter inspection、help 行为和退出码。

## 命令面

`docnav` 提供以下命令：

```text
docnav outline <path> [document operation common flags] [--pagination enabled|disabled] [--page 1] [--limit 6000]
docnav read <path> --ref <ref> [document operation common flags] [--pagination enabled|disabled] [--page 1] [--limit 6000]
docnav find <path> --query <text> [document operation common flags] [--pagination enabled|disabled] [--page 1] [--limit 6000]
docnav info <path> [document operation common flags]
docnav init [--project-config <path>]
docnav doctor [--project-config <path>] [--user-config <path>]
docnav config inspect [--project-config <path>] [--user-config <path>]
docnav adapter list
docnav version
```

Document operation common flags: `[--project-config <path>] [--user-config <path>] [--adapter <adapter-id>] [--invocation-log <path>] [--invocation-log-content-root <path>] [--output readable-view|readable-json|protocol-json]`.

`outline`、`read`、`find` 和 `info` 是 document operation。`init`、`doctor`、`config`、`adapter list` 和 `version` 是 core CLI 命令，不产生 document operation request。

## Document operation 执行

Document operation 按以下顺序执行：

1. 解析 command、subcommand、固定 positional、help 和显式 argv token。
2. 确定项目根。
3. 规范化 document path，并检查文件可访问性。
4. 解析 invocation logging 的 core-owned CLI/config surface；未显式启用时不得创建日志 sink 或 content capture 文件。
5. 解析并校验当前 operation 使用的 CLI 参数。
6. 选择 adapter。
7. 对 `outline` 解析 navigation-owned `outline_mode` selectors。
8. 构造内部 document operation request。
9. 调用选定 adapter 的 operation handler，或在 `outline_mode = "unstructured_full"` 时进入 navigation pre-dispatch full-read path。
10. 输出结果，并映射进程退出码。

非法 CLI 输入必须在 adapter 选择和 document operation handler 调用前失败。未知 argv、多余 positional、当前 operation 不使用的已知参数、缺少必需 path/ref/query、非法 page、非法 limit 或非法 output 都是 input failure；当前 operation 不使用的参数不触发其它 operation 的 eager validation。

路径不存在、不可读或无法规范化时返回文档路径错误，不能调用 adapter layer。

## Invocation logging

Document operation `outline`、`read`、`find` 和 `info` 支持 runtime invocation logging。该能力默认关闭；关闭时不得新增 stdout/stderr 输出、protocol/readable 字段、adapter handler payload 或日志文件副作用。

启用 surface：

| Surface | 含义 |
| --- | --- |
| `--invocation-log <path>` | 为本次 invocation 显式启用 JSONL invocation log，并把 event 追加写入该文件路径。 |
| `--invocation-log-content-root <path>` | 单独显式启用 content capture，并把正文文件写入该 root；只有同时启用 `--invocation-log` 时有效。 |
| `invocation_log.enabled` | 配置文件中的显式启用开关；必须与 `invocation_log.path` 一起出现才启用日志。 |
| `invocation_log.path` | 配置文件中的 JSONL log file path。 |
| `invocation_log.content_capture.enabled` | 配置文件中的 content capture 显式启用开关；必须与 `invocation_log.content_capture.root` 一起出现才启用正文捕获。 |
| `invocation_log.content_capture.root` | 配置文件中的 content capture root path。 |

CLI 显式 path 优先于配置文件 path；配置文件来源仍遵循 project config 优先于 user config。未提供 CLI path 且配置未同时给出 `enabled: true` 与 `path` 时，不存在默认日志路径。未提供 content capture root 或 content capture 未显式启用时，不写正文文件。

Invocation log path 和 content capture root 都按普通文件系统 path 规范化；相对路径以当前 invocation 的 project context 为基准。Log path 指向 JSONL file；content capture root 指向目录。Content capture 文件必须位于 root 下的日期目录中，相对路径格式固定为 `<YYYY-MM-DD>/sha256-<content_hash>.content`，其中 `content_hash` 是小写 64 位十六进制 SHA-256。

Invocation log 默认记录 metadata-only event：schema version、timestamp、event name、request/correlation id、operation、selected adapter id when available、duration、operation/output status、响应大小摘要、bounded diagnostic summary、path display 和 bounded query/ref summary。
Path display 默认记录 project-relative path；项目根外文档记录规范化绝对 path 的 bounded display 和 hash metadata。
Query/ref 不作为跨层稳定身份记录，默认只记录 presence、length、hash 或 bounded preview，不记录无界原始值。

主 invocation log 不得 inline 完整 document content、完整 `RequestEnvelope` / `ProtocolResponse`、完整 diagnostic/debug output、环境变量或 secrets。涉及正文内容时，操作结果事件只记录 `hash_algorithm: "sha256"`、小写 64 位十六进制 `content_hash`、content type、size metadata 和可选 bounded summary。Content capture 必须由独立 surface 显式开启；开启后主日志追加 `content_captured` 或 `content_capture_failed` event，正文 bytes 不进入 document stdout、protocol/readable output 或主操作结果 event。

## Parser 与 help

Rust CLI argv 结构解析以 `clap` 或 `clap` builder API 为基础。CLI 使用 parser 描述 command shape、subcommand、固定 positional、枚举值和 help。

Root help 和子命令 help 只输出 help 文本，不执行项目解析、配置读取、adapter 选择或 document operation。

Help 文本必须只在 documented 支持的 command surface 展示 `--project-config <path>` / `--user-config <path>`：document operation、`config` 和 `doctor` 展示两者，`init` 只展示 `--project-config <path>`。

非 document command 使用自己的 command shape 解析 argv。无关 argv 按该命令的输入错误处理，不构造 document operation request。

未知 config path flag、config path flag 缺少 value，或在未文档化支持该 flag 的命令上使用 config path flag，都是 strict input failure。该失败发生在读取 config source、运行 `config`/`init`/`doctor` 目标逻辑或分派 document operation 之前；非法 argv 不得被忽略，也不得被解释为 config JSON 字段。

## 项目根与路径

`docnav` 按以下顺序确定项目根：

1. 从启动 cwd 向上查找最近的 `.docnav/`。
2. 未找到时使用启动 cwd。

Document operation、`init`、`doctor` 和 `config` 命令使用该项目根解析项目配置和项目上下文。

`docnav` 接受项目根内外的可访问文件路径。相对 path 基于启动 cwd 解析。`document.path` 必须使用 `/`：项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。

## 配置文件路径

`--project-config <path>` 和 `--user-config <path>` 选择本次 invocation 使用的 exact config JSON file path。显式 path 是文件路径，不是目录；core 不会对显式 path 自动追加 `.docnav/docnav.json`。

支持范围：

- Document operation `outline`、`read`、`find` 和 `info` 支持 `--project-config <path>` 与 `--user-config <path>`，并把解析后的 project/user config source descriptor 交给 `docnav-navigation`。
- `docnav config inspect` 支持 `--project-config <path>` 与 `--user-config <path>`，并只用它们选择本次要检查的 project/user config source。
- `docnav doctor` 支持 `--project-config <path>` 与 `--user-config <path>`。
- `docnav init` 只支持 `--project-config <path>`；`--user-config` 不是 `init` 的 documented input。
- `adapter list`、`version`、root help 和 adapter inspection 命令不支持 config path flag。

未传显式 path 时，core 使用默认路径：

1. User config path 先使用 `DOCNAV_CONFIG_DIR/docnav.json`；`DOCNAV_CONFIG_DIR` 是目录语义。环境变量不存在时，使用平台用户默认位置下的 `.docnav/docnav.json`。
2. Project config path 使用当前 project context 下的 `.docnav/docnav.json`。project context 仍按“项目根与路径”确定，`--project-config` 不改变 project root discovery。

显式 path flag 和默认路径解析只选择配置文件来源，不改变 navigation 参数来源优先级；配置文件内部值仍按 `explicit > project > user > built_in` 解析。

Runtime invocation logging 可读取同一 project/user config 文件中的 `invocation_log` section，但该 section 是 core-owned runtime 配置，不是 adapter native option、navigation default 或 protocol request argument。日志启用、sink/path 和 content capture root 的错误归属由本文件拥有；navigation 参数来源合并不得把 `invocation_log.*` 写入 `RequestEnvelope.arguments`。

## 配置命令

`docnav config inspect` 是长期保留的唯一 config 子命令。它是只读 source inspection command，不是 config editor：

- 读取 selected project/user config source，报告每个 source 的 scope、selected path、path origin、exists/missing/load state、source summary 和 source-attributed validation diagnostics。
- 通过 owner-provided parameter aggregation metadata 产出的 config-source projection 校验可表达字段，展示当前 selected sources 中可解析出的参数事实；core CLI 不重新定义 output enum、positive integer、outline selector 或 adapter native option 的字段语义。
- Adapter native option 的持久 config source path 固定为 `options.<adapter-id>.<option-key>`；`<adapter-id>` 使用当前 core release static registry 中的 adapter id，不使用 alias。旧裸 `options.<option-key>` 不兼容、不迁移，只按普通 unknown/invalid config path 处理。
- `config inspect` 不修改任何 config file，不接受 key/value edit，不删除字段，不提供 single-key get/list 语义。
- `config inspect` 不产生 document operation request，不构造 adapter operation arguments，不调用 probe 或 adapter handler，也不声称某个 adapter option 已 dispatch。Selected adapter/operation validation 仍由 [Navigation Input Resolution](navigation-input-resolution.md) 拥有。

`docnav config get`、`docnav config set`、`docnav config unset` 和 `docnav config list` 是 breaking legacy surface，不再是 accepted subcommand；调用这些名称必须经 normal CLI parse/error boundary 拒绝，且不得修改 config file。

`outline.mode_rules[]` 和 `outline.auto_full_read.thresholds[]` 只能通过 config source 参与 `outline_mode` resolution。CLI 不提供 public outline-mode override flag。

## 内置 adapter 检查

`docnav adapter list` 展示当前 release 编译进 static registry 的 adapter metadata，例如 adapter id、名称、版本、core-owned implementation source、支持格式、扩展名、content type 和 operation metadata。Adapter-owned metadata 来自 registered adapter definition；implementation source 由 core static registry 记录。

默认 adapter 命令面只包含 `docnav adapter list`。

`docnav init --project-config <path>` 创建或保留 selected project config file；未传时创建或保留当前 project context 的 `.docnav/docnav.json`。

`docnav doctor` 检查 selected project/user config files、static registry 和 core release 内置 adapter layer 可用性。doctor 可以验证静态 descriptor 与 linked handler 是否一致；修复建议必须落在当前配置、static registry 或 linked adapter layer 边界内。

## adapter 执行入口

默认 CLI 的 adapter 执行入口是 core-linked library handle。core CLI 通过内部 navigation layer 调用选定 adapter 的 `outline/read/find/info` operation handlers。

## 退出码

CLI 使用以下进程退出码：

- 成功退出 `0`。
- 输入错误退出 `2`。
- 文档、ref 或格式错误退出 `3`。
- protocol 或 adapter layer 错误退出 `4`。
- 内部错误退出 `1`。

Invocation logging 写入失败、content capture root 不可写或单条 event serialization/append 失败不得改变原 document operation 的退出码。若需要报告日志失败诊断，只能使用不会污染 machine-readable stdout 的 bounded 通道；`protocol-json` 和 `readable-json` stdout 仍必须只包含对应 document output JSON 值。
