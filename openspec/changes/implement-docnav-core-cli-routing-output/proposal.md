**一句话核心：实现 `docnav` 核心 CLI 的路径解析、adapter 预选/继续遍历、invoke 调用和输出映射，跑通 `outline -> ref -> read` 首条端到端链路。**

## Why

`docnav` 是 CLI-first 的核心 router/manager。它不解析具体格式内容，但负责把用户命令解析为稳定的 adapter 调用：发现项目根、读取配置、解析默认参数、选择 adapter、启动 invoke、校验协议响应、映射输出模式和错误。

Markdown adapter 完成后，核心 CLI 需要把用户可执行的 `docnav outline/read/find/info` 命令稳定接入 adapter，并产出 text、readable-json 和 protocol-json 三类输出。

## What Changes

- 实现文档操作命令：`docnav outline/read/find/info`。
- 实现基础管理命令：`docnav init/doctor/version/config get|set|unset|list`。
- 实现兼容性参数处理：未知 flag、多余 positional 和当前 operation 不使用的已知 flag 不阻断执行，生成列明具体 ignored token、kind 和 reason 的 warning 后忽略；未知 flag 不吞后续 token；已知有值 flag 固定消费紧跟 token；已知必需参数缺失、已知 flag 缺少值或值非法仍返回稳定错误。
- 实现项目根发现、任意可访问文件 path 规范化和最终 core 通用参数解析。
- `docnav` 只处理 path、ref、query、page、limit_chars、output 和 adapter；page 省略时写入 `1`，limit_chars 解析为有限正整数，manifest 不提供默认参数，core 不合成格式 options。
- 实现 adapter 选择流程：
  1. 调用方传入 `--adapter <adapter-id>` 时，该 id 是预选 adapter。
  2. 未传 `--adapter` 时，项目/用户配置的 `defaults.adapter` 先参与预选；配置缺失时，core 基于候选 manifest 的 `formats[].extensions[]` 等轻量信息推断一个预选 adapter；无法推断时预选为空。
  3. 对预选 adapter 执行解析、manifest 当前 schema/语义校验和 probe；probe 成功即选中。
  4. 预选缺失、adapter 记录解析失败或 probe 返回有效 `supported: false` 时，将该候选记为失败证据，并按 registry 顺序继续遍历候选，返回第一个 probe 成功的 adapter。
  5. 任一候选的 manifest/probe 输出不符合当前 schema 或语义校验时直接返回 adapter/protocol 错误；选中 adapter 的 invoke 输出不符合当前 schema 或语义校验时也直接失败。
- 本 change 不做协议版本协商或兼容迁移，只接受当前 schema 和当前语义契约。
- 调用选中 adapter 的 `invoke`，校验 protocol 响应，并映射为默认阅读文本、readable-json 或 protocol-json。
- warning 按输出模式承载：text 输出在正常阅读文本后拼接 warning，readable-json 输出增加 `warnings` 数组；protocol-json stdout 保持 schema-valid protocol envelope 且不增加 `warnings` 字段，CLI warning 只写 stderr。
- `--output protocol-json` 对 core 自身产生的错误也输出 protocol failure envelope；阅读输出保留精简错误语义。
- 保留 adapter 返回的 ref、display、content、content_type、cost 和 page 业务字段。
- 本 change 只使用可替换的简化 adapter 记录读取接口，不实现正式 adapter 安装/更新/移除、黑白名单、Markdown parser 或 MCP bridge。

## Capabilities

### New Capabilities

- `docnav-core-cli-routing-output-implementation`: 实现核心 CLI 文档操作、adapter 选择、invoke 调用、输出层映射、配置命令和错误映射。

### Modified Capabilities

- 无。

## Impact

- 影响核心可执行文件：`docnav`。
- 影响配置和发现模型：核心默认参数、项目/用户配置读取、简化 adapter 记录读取。
- 影响端到端测试：`docnav outline -> ref -> read`、显式/推断/遍历 adapter 选择、page 继续读取、输出层差异和稳定错误 code。
