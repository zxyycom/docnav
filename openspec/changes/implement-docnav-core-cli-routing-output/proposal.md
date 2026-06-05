**一句话核心：实现 `docnav` 核心 CLI，把 path、format、adapter 选择、invoke 和阅读输出统一串成首条端到端链路。**

## Why

`docnav` 是 CLI-first 的核心 router/manager，负责项目根、配置、默认参数、adapter 选择、invoke 启动、协议校验、输出模式和错误映射。Markdown adapter 完成后，需要核心 CLI 将用户命令稳定映射到 adapter，并产出 text、readable-json 和 protocol-json。

## What Changes

- 新增或完善 `docnav outline/read/find/info/init/doctor/version/config` 核心命令。
- 实现项目根发现、path 规范化、项目边界检查和最终有限参数解析。
- 实现 adapter 选择顺序：显式 format/content type 校验、扩展名候选校验、全量 probe。
- 调用选中 adapter 的 `invoke`，校验 protocol 响应，并映射为默认阅读文本、readable-json 或 protocol-json。
- 保留 adapter 返回的 ref、display、content、content_type、cost 和 page。
- 非目标：本 change 不实现正式 adapter 安装/更新/移除算法，不实现 Markdown parser，不实现 MCP bridge。

## Capabilities

### New Capabilities

- `docnav-core-cli-routing-output-implementation`: 实现核心 CLI 文档操作、adapter 选择、invoke 调用、输出层映射和错误映射。

### Modified Capabilities

- 无。

## Impact

- 影响核心可执行文件：`docnav`。
- 影响配置和发现模型：核心默认参数、项目/用户配置读取、adapter 记录读取。
- 影响端到端测试：`docnav outline -> ref -> read`、format 选择、page 继续读取和输出层差异。
