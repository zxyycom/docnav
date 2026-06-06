**一句话核心：实现 `docnav` 核心 CLI 的路径解析、adapter 预选/回退、invoke 调用和输出映射，跑通 `outline -> ref -> read` 首条端到端链路。**

## Why

`docnav` 是 CLI-first 的核心 router/manager。它不解析具体格式内容，但负责把用户命令解析为稳定的 adapter 调用：发现项目根、读取配置、解析默认参数、选择 adapter、启动 invoke、校验协议响应、映射输出模式和错误。

Markdown adapter 完成后，核心 CLI 需要把用户可执行的 `docnav outline/read/find/info` 命令稳定接入 adapter，并产出 text、readable-json 和 protocol-json 三类输出。

## What Changes

- 实现文档操作命令：`docnav outline/read/find/info`。
- 实现基础管理命令：`docnav init/doctor/version/config get|set|unset|list`。
- 实现项目根发现、任意可访问文件 path 规范化和最终有限参数解析。
- 实现 adapter 选择流程：
  1. 有 `--adapter <adapter-id>` 时，将该 id 作为预选 adapter。
  2. 没有 `--adapter` 时，由 core 使用扩展名等轻量规则推断预选 adapter；无法推断时预选为空。
  3. 对预选 adapter 执行 probe，成功即选中。
  4. 预选缺失、解析失败或 probe 失败时，按 registry 顺序遍历候选，返回第一个 probe 成功的 adapter。
- 调用选中 adapter 的 `invoke`，校验 protocol 响应，并映射为默认阅读文本、readable-json 或 protocol-json。
- 保留 adapter 返回的 ref、display、content、content_type、cost 和 page 业务字段。
- 本 change 只使用可替换的简化 adapter 记录读取接口，不实现正式 adapter 安装/更新/移除、黑白名单、版本化 registry、Markdown parser 或 MCP bridge。

## Capabilities

### New Capabilities

- `docnav-core-cli-routing-output-implementation`: 实现核心 CLI 文档操作、adapter 选择、invoke 调用、输出层映射、配置命令和错误映射。

### Modified Capabilities

- 无。

## Impact

- 影响核心可执行文件：`docnav`。
- 影响配置和发现模型：核心默认参数、项目/用户配置读取、简化 adapter 记录读取。
- 影响端到端测试：`docnav outline -> ref -> read`、显式/推断/遍历 adapter 选择、page 继续读取、输出层差异和稳定错误 code。
