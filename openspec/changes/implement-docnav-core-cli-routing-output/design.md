**一句话核心：核心 CLI 把用户命令解析成确定的 adapter invoke 请求，并把 adapter 协议结果映射成阅读输出。**

## Context

`docnav` 是所有接入方式共享的核心契约。它不解析格式内容，而是负责命令解析、项目根、path、配置、adapter 选择、默认参数、invoke 进程、协议校验、输出层和错误映射。

本 change 的目标是让 Markdown adapter 可以通过核心 CLI 被稳定调用，并让后续 MCP bridge、正式 adapter 管理和其它格式 adapter 复用同一条路由与输出链路。

## Goals / Non-Goals

**Goals:**

- 实现 `docnav outline/read/find/info` 文档操作。
- 实现 `init`、`doctor`、`version` 和 `config get|set|unset|list` 的核心 CLI 基础。
- 实现由 `--adapter`、core 简易推断和 registry 遍历组成的 adapter 选择。
- 实现 text、readable-json 和 protocol-json 输出。
- 保证 `outline -> ref -> read` 端到端链路可运行。

**Non-Goals:**

- 不实现 Markdown parser。
- 不实现正式 adapter install/update/remove/list 的完整算法。
- 不实现 MCP transport。

## Decisions

1. core CLI 在启动 invoke 前完成自身入口的全部默认参数解析。
   - 从 `docnav` 入口省略 page 时，core CLI 在 invoke 请求中显式传入 `page: 1`。
   - adapter 直接 CLI 也必须支持省略 page，并在自身入口解析为同一初始页；core CLI 的默认值解析不替代 adapter 直接 CLI 的参数解析。
   - limit_chars 从显式参数、项目配置、用户配置和内置默认值解析为有限正整数。

2. path 规范化支持任意可访问文件路径。
   - 相对 path 基于启动 cwd 解析；项目根只用于读取 `docnav` 配置和临时 adapter 记录。
   - adapter 子进程 cwd 设置为项目根；没有项目根时使用启动 cwd。
   - `document.path` 使用 `/`；项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。
   - `docnav` 不因 path 位于项目根外而拒绝请求，只拒绝不存在、不可读或无法规范化的输入。

3. adapter 选择先确定预选 adapter，再按统一函数回退遍历。
   - 第一步：确定 `preselected_adapter_id`。
     - 若调用方提供 `--adapter <adapter-id>`，该 id 就是预选 adapter。
     - 若调用方未提供 `--adapter`，core 使用扩展名等轻量规则推断预选 adapter id；无法推断时预选为空。
   - 第二步：校验预选 adapter。
     - 若预选 adapter 存在，`docnav` 解析该 adapter 并执行 probe。
     - probe 成功即选中该 adapter。
   - 第三步：预选缺失、解析失败或 probe 失败时，调用 registry 遍历函数。
     - 遍历函数接收已尝试 adapter id 集合，按 registry 顺序跳过已尝试项。
     - 遍历函数返回第一个 probe 成功的 adapter，不要求检测多个成功候选。
   - 所有候选均失败时返回 `FORMAT_UNKNOWN` 和候选证据；本 change 不实现 `FORMAT_AMBIGUOUS` 检测。

4. 简化 adapter 记录只服务当前实现链路。
   - 本 change 使用相对命令路径记录，使 `docnav` 能找到 adapter 可执行文件。
   - 读取接口需要可被正式 adapter 管理 registry 替换。
   - 正式 allowlist、denylist、adapter id 到多版本命令路径映射、当前版本选择和安装记录属于 adapter 管理 change。

5. 输出映射分层处理。
   - `protocol-json` 输出完整原始协议 envelope。
   - text/readable-json 输出阅读层结果，不包含 envelope。
   - read 的 readable 输出保留 `content_type`。

6. 错误映射保持 code 稳定、展示可配置。
   - 稳定错误 code 不受文本模板配置影响。
   - CLI 退出码按主规范映射。
   - `config get` 的 key 不存在时返回 `INVALID_REQUEST`。

## Risks / Trade-offs

- [adapter 选择流程较复杂] → 用测试覆盖 `--adapter` 预选、core 推断预选、预选失败回退遍历和全失败。
- [阅读输出与 protocol 输出漂移] → 对同一 fixture 同时验证 protocol-json 和 readable-json 业务语义一致。
- [配置影响稳定字段] → 配置只允许影响文本模板、guidance、usage 和错误建议。
- [项目外 path 与项目配置并存] → 测试项目根内相对 path、项目根外绝对 path 和从不同 cwd 启动的相对 path。

## Migration Plan

1. 在协议/SDK和 Markdown adapter 完成后实现核心 CLI。
2. 先支持简单相对命令路径 adapter 记录，使路由链路可测。
3. adapter 管理 change 完成后切换到正式安装记录读取。

## Open Questions

- 正式 adapter registry 的 allowlist、denylist、版本选择和用户级命令路径存储由 adapter 管理 change 完成；本 change 只需要定义可被替换的读取接口。
