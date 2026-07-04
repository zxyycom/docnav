# CLI

本文定义 `docnav` CLI 的命令面、命令解析、项目根与路径处理、配置命令、内置 adapter inspection、help 行为和退出码。

## 命令面

`docnav` 提供以下命令：

```text
docnav outline <path> [--adapter <adapter-id>] [--pagination enabled|disabled] [--page 1] [--limit 6000] [--output readable-view|readable-json|protocol-json]
docnav read <path> --ref <ref> [--adapter <adapter-id>] [--pagination enabled|disabled] [--page 1] [--limit 6000] [--output readable-view|readable-json|protocol-json]
docnav find <path> --query <text> [--adapter <adapter-id>] [--pagination enabled|disabled] [--page 1] [--limit 6000] [--output readable-view|readable-json|protocol-json]
docnav info <path> [--adapter <adapter-id>] [--output readable-view|readable-json|protocol-json]
docnav init
docnav doctor
docnav config get|set|unset|list [--user] [--path <path>] [--operation outline|read|find|info]
docnav adapter list
docnav version
```

`outline`、`read`、`find` 和 `info` 是 document operation。`init`、`doctor`、`config`、`adapter list` 和 `version` 是 core CLI 命令，不产生 document operation request。

## Document operation 执行

Document operation 按以下顺序执行：

1. 解析 command、subcommand、固定 positional、help 和显式 argv token。
2. 确定项目根。
3. 规范化 document path，并检查文件可访问性。
4. 解析并校验当前 operation 使用的 CLI 参数。
5. 选择 adapter。
6. 对 `outline` 解析 navigation-owned `outline_mode` selectors。
7. 构造内部 document operation request。
8. 调用选定 adapter 的 operation handler，或在 `outline_mode = "unstructured_full"` 时进入 navigation pre-dispatch full-read path。
9. 输出结果，并映射进程退出码。

非法 CLI 输入必须在 adapter 选择和 document operation handler 调用前失败。未知 argv、多余 positional、当前 operation 不使用的已知参数、缺少必需 path/ref/query、非法 page、非法 limit 或非法 output 都是 input failure；当前 operation 不使用的参数不触发其它 operation 的 eager validation。

路径不存在、不可读或无法规范化时返回文档路径错误，不能调用 adapter layer。

## Parser 与 help

Rust CLI argv 结构解析以 `clap` 或 `clap` builder API 为基础。CLI 使用 parser 描述 command shape、subcommand、固定 positional、枚举值和 help。

Root help 和子命令 help 只输出 help 文本，不执行项目解析、配置读取、adapter 选择或 document operation。

非 document command 使用自己的 command shape 解析 argv。无关 argv 按该命令的输入错误处理，不构造 document operation request。

## 项目根与路径

`docnav` 按以下顺序确定项目根：

1. 从启动 cwd 向上查找最近的 `.docnav/`。
2. 未找到时使用启动 cwd。

Document operation、`init`、`doctor` 和 `config` 命令使用该项目根解析项目配置和项目上下文。

`docnav` 接受项目根内外的可访问文件路径。相对 path 基于启动 cwd 解析。`document.path` 必须使用 `/`：项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。

## 配置命令

`docnav config get|set|unset|list` 是 core CLI 命令族。

- `docnav config set` 和 `unset` 默认写项目配置。
- `docnav config set --user` 和 `unset --user` 写用户配置。
- `config get` 的 key 不存在时返回 `INVALID_REQUEST`。
- `config list` 不带 path 时列出 core 配置域当前生效值。
- `config list --path <path> [--operation outline|read|find|info]` 解析 document context，并展示该 path 触发的 adapter、参数来源和最终值。
- `config` 命令不产生 document operation request。

`outline.mode_rules[]` 和 `outline.auto_full_read.thresholds[]` 只能通过 config source 参与 `outline_mode` resolution。CLI 不提供 public outline-mode override flag。

## 内置 adapter 检查

`docnav adapter list` 展示当前 release 编译进 static registry 的 adapter metadata，例如 adapter id、名称、版本、支持格式、扩展名、content type 和 operation metadata。

默认 adapter 命令面只包含 `docnav adapter list`。

`docnav doctor` 检查项目配置、用户配置、static registry 和 core release 内置 adapter layer 可用性。doctor 可以验证静态 descriptor 与 linked handler 是否一致；修复建议必须落在当前配置、static registry 或 linked adapter layer 边界内。

## adapter 执行入口

默认 CLI 的 adapter 执行入口是 core-linked library handle。core CLI 通过内部 navigation layer 调用选定 adapter 的 `outline/read/find/info` operation handlers。

## 退出码

CLI 使用以下进程退出码：

- 成功退出 `0`。
- 输入错误退出 `2`。
- 文档、ref 或格式错误退出 `3`。
- protocol 或 adapter layer 错误退出 `4`。
- 内部错误退出 `1`。
