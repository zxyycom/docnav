**一句话核心：核心 CLI 把用户命令解析成确定的 adapter invoke 请求，并把 adapter 协议结果映射成阅读输出。**

## Context

`docnav` 是所有接入方式共享的核心契约。它不解析格式内容，而是负责命令解析、项目根、path、配置、adapter 选择、默认参数、invoke 进程、协议校验、输出层和错误映射。

本 change 的目标是让 Markdown adapter 可以通过核心 CLI 被稳定调用，并让后续 MCP bridge、正式 adapter 管理和其它格式 adapter 复用同一条路由与输出链路。

## Goals / Non-Goals

**Goals:**

- 新增独立的 `docnav` 核心 Rust crate 和可执行入口。
- 实现 `docnav outline/read/find/info` 文档操作。
- 实现 `init`、`doctor`、`version` 和 `config get|set|unset|list` 的核心 CLI 基础。
- 实现由 `--adapter`、core 简易推断和 registry 遍历组成的 adapter 选择。
- 实现 text、readable-json 和 protocol-json 输出。
- 保证 `outline -> ref -> read` 端到端链路可运行。

**Non-Goals:**

- Markdown parser 由格式 adapter 实现；core CLI 只调用 adapter 并校验协议结果。
- 正式 adapter install/update/remove/list 由 adapter 管理 change 实现；本 change 只提供可替换的临时 registry 读取接口。
- MCP transport 由 MCP bridge change 实现；本 change 只交付 `docnav` CLI。

## Decisions

1. core CLI 以独立 workspace crate 落地。
   - 新增 `crates/docnav` workspace member。
   - package/bin 名称均为 `docnav`。
   - `docnav` crate 只承担核心 CLI、配置、registry 读取、adapter 子进程调用、协议响应校验和输出映射；格式解析由选中的 adapter 完成。
   - 核心 CLI smoke 通过真实 `docnav` 二进制和真实 adapter 子进程验证进程边界。

2. core CLI 在启动 invoke 前完成自身入口的全部默认参数解析。
   - 从 `docnav` 入口省略 page 时，core CLI 在 invoke 请求中显式传入 `page: 1`。
   - adapter 直接 CLI 也必须支持省略 page，并在自身入口解析为同一初始页；core CLI 的默认值解析不替代 adapter 直接 CLI 的参数解析。
   - limit_chars 从显式参数、项目配置、用户配置和内置默认值解析为有限正整数。
   - `docnav` 只处理 core 通用参数：path、ref、query、page、limit_chars、output 和 adapter。
   - adapter 专属 CLI flag 和格式 options 由对应 adapter 或后续格式能力定义；core CLI 在本 change 中不读取 manifest 默认参数，也不合成格式 options。
   - 未知参数和多余参数按兼容性输入处理：生成列明具体 token 的 warning 后忽略。
   - warning 按输出模式承载：text 输出在正常结果后拼接 warning；readable-json 输出增加 `warnings` 数组；protocol-json stdout 保持 schema-valid protocol envelope，CLI warning 只写 stderr。
   - 已知 flag 需要值时，紧跟该 flag 的下一个 token 就是值，即使它以 `--` 开头；只有不存在下一个 token 时才返回缺值 `INVALID_REQUEST`。
   - 已知必需参数缺失、已知 flag 缺少值或值非法时返回 `INVALID_REQUEST`。

3. core 配置先实现 MVP，避免提前固化完整配置系统。
   - 项目配置文件为 `<project-root>/.docnav/docnav.json`。
   - 用户配置文件优先使用 `DOCNAV_CONFIG_DIR/docnav.json`；未设置时使用平台用户配置目录下的 `docnav/docnav.json`。
   - 本 change 只支持 `defaults.adapter`、`defaults.limit_chars` 和 `defaults.output` 三个 key。
   - `defaults.adapter` 只在未传 `--adapter` 时作为预选 adapter；优先级高于 core 简易推断。
   - page 不可配置，省略时固定为 `1`。
   - `config set` 和 `config unset` 默认写项目配置；传入 `--user` 时写用户配置。
   - `config get` 读取当前生效值，按项目配置、用户配置和内置默认值合并。
   - `config list` 不带 path 时输出 core 配置域中所有支持 key 的当前生效值及来源。
   - `config list --path <path> [--operation outline|read|find|info]` 先按文档命令规则解析 path、选择 adapter 和读取 manifest，再输出该文档上下文下的生效配置、选中 adapter 和最终 core 默认参数；operation 省略时使用 `outline`。
   - `config get|set|unset|list` 只管理上述 key；未知 key 或不存在 key 返回 `INVALID_REQUEST`。

4. core 基础管理命令提供可验证 MVP 行为。
   - `init` 创建 `<project-root>/.docnav/docnav.json`，重复执行保持幂等。
   - `version` 输出 `docnav` crate 版本。
   - `doctor` 检查项目配置、用户配置、临时 adapter registry 和已记录 adapter CLI 的可用性。
   - `doctor` 执行全部可运行检查，并以 `checks` 数组输出每个检查项的 status、target 和 message；有失败项时非零退出，退出码按最严重失败项映射。

5. path 规范化支持任意可访问文件路径。
   - 相对 path 基于启动 cwd 解析；项目根只用于读取 `docnav` 配置和临时 adapter 记录。
   - adapter 子进程 cwd 设置为项目根；没有项目根时使用启动 cwd。
   - `document.path` 使用 `/`；项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。
   - `docnav` 不因 path 位于项目根外而拒绝请求，只拒绝不存在、不可读或无法规范化的输入。

6. adapter 选择先确定预选 adapter，再按统一函数继续遍历。
   - 第一步：确定 `preselected_adapter_id`。
     - 若调用方提供 `--adapter <adapter-id>`，该 id 就是预选 adapter。
     - 若调用方未提供 `--adapter`，项目配置 `defaults.adapter` 优先于用户配置 `defaults.adapter`。
     - 若调用方和配置都未指定 adapter，core 使用 manifest `formats[].extensions[]` 等轻量规则推断预选 adapter id；无法推断时预选为空。
   - 第二步：校验预选 adapter。
     - 若预选 adapter 存在，`docnav` 解析该 adapter，执行 `manifest --output protocol-json`，校验 manifest schema、manifest 语义和当前 CLI 需要的当前契约字段，再执行 probe。
     - probe 成功即选中该 adapter。
     - 若预选 adapter 的 registry 记录解析失败，记录候选证据；显式 `--adapter` 场景同时生成 warning，然后继续 registry 遍历。
     - 若 probe 返回符合当前 schema/语义的 `supported: false`，记录 `PROBE_UNSUPPORTED` 候选证据；显式 `--adapter` 场景同时生成 warning，然后继续 registry 遍历。
     - 若 manifest/probe 输出不符合当前 schema 或语义校验，`docnav` 直接返回 adapter/protocol 错误，不继续遍历。
   - 第三步：预选缺失、registry 记录解析失败或有效 probe 不支持时，调用 registry 遍历函数。
     - 遍历函数接收已尝试 adapter id 集合，按 registry 顺序跳过已尝试项。
     - 遍历函数对每个候选执行 registry 记录解析、manifest schema 校验、当前契约语义校验和 probe。
     - 遍历函数遇到有效 `supported: false` 时记录候选证据并继续；遇到第一个 probe 成功的 adapter 时立即返回，不检测多个成功候选。
   - 本 change 不做协议版本协商；`docnav` 只接受当前 schema 和语义契约。
   - manifest/probe 输出字段缺失、字段类型不符、schema 或 semantic validation 失败表示 adapter 当前契约不一致，直接返回 adapter/protocol 错误，不继续尝试其它 adapter。
   - invoke 输出字段缺失、字段类型不符、operation/result shape 不匹配或 schema/semantic validation 失败表示选中 adapter 调用失败，直接返回 adapter/protocol 错误。
   - 所有候选均失败时返回 `FORMAT_UNKNOWN` 和候选证据；本 change 不实现 `FORMAT_AMBIGUOUS` 检测。
   - 候选证据使用稳定 JSON 数组，每项形状为 `{ "adapter_id": string, "stage": "resolve"|"probe", "code": string, "reason": string, "details": object }`。`code` 至少覆盖 `ADAPTER_NOT_FOUND` 和 `PROBE_UNSUPPORTED`。

7. 简化 adapter 记录只服务当前实现链路。
   - 本 change 使用项目级 `<project-root>/.docnav/adapters.json` 作为临时 registry。
   - registry JSON 形状为 `{ "version": 1, "adapters": [{ "id": "...", "command": "relative/path/to/adapter" }] }`。
   - `command` 必须是相对项目根的命令路径；绝对路径、本机用户目录、来源 URL、fingerprint 和版本选择不属于本 change。
   - `adapters` 数组顺序就是 registry 遍历顺序。
   - registry 缺失表示没有显式候选；registry JSON 损坏、字段缺失或相对命令非法返回 `INVALID_REQUEST`。
   - 读取接口需要可被正式 adapter 管理 registry 替换。
   - 正式 allowlist、denylist、adapter id 到命令路径映射和安装记录属于 adapter 管理 change。

8. 输出映射分层处理。
   - `protocol-json` 输出完整原始协议 envelope。
   - core 在启动 invoke 前产生的错误也按当前 operation 输出 protocol failure envelope；无法确定 operation 时使用 `operation: null`。
   - text/readable-json 输出阅读层结果，不包含 envelope。
   - 非致命 warning 必须进入所选输出模式：text 在正常阅读文本后拼接 warning，readable-json 输出增加 `warnings` 数组；protocol-json stdout 不增加 `warnings` 字段，warning 写入 stderr。
   - read 的 readable 输出保留 `content_type`。

9. 错误映射保持 code 稳定、展示可配置。
   - 稳定错误 code 不受文本模板配置影响。
   - CLI 退出码按主规范映射。
   - `config get` 的 key 不存在时返回 `INVALID_REQUEST`。
   - adapter stdout 不是单一 JSON、protocol response schema 或语义校验失败、operation/request_id 不匹配、adapter 非零退出且无法读取结构化错误时，映射为 protocol 或 adapter 进程错误。

## Risks / Trade-offs

- [adapter 选择流程较复杂] → 用测试覆盖 `--adapter` 预选、配置预选、core 推断预选、有效 probe 不支持后继续遍历、契约校验失败直接返回和全失败。
- [显式 adapter 解析失败或有效 probe 不支持后继续遍历可能让用户误解] → 对该候选生成包含 adapter id、阶段和原因的 warning，并按 text/readable-json/protocol-json 输出模式边界承载。
- [临时 registry 可能和正式 registry 漂移] → 将读取逻辑封装为可替换接口，临时文件不保存正式管理才拥有的 version/source/fingerprint。
- [阅读输出与 protocol 输出漂移] → 对同一 fixture 同时验证 protocol-json 和 readable-json 业务语义一致。
- [配置影响稳定字段] → 本 change 的配置只允许影响预选 adapter、limit_chars 和 output；page 固定由 CLI 默认值解析，错误 code 和当前契约字段类型保持不变。
- [项目外 path 与项目配置并存] → 测试项目根内相对 path、项目根外绝对 path 和从不同 cwd 启动的相对 path。
- [兼容性参数处理隐藏调用错误] → 对每个忽略的未知或多余参数生成包含 token 的 warning，并测试 warning 不改变成功退出码。

## Migration Plan

1. 在协议/SDK和 Markdown adapter 完成后实现核心 CLI。
2. 先支持简单相对命令路径 adapter 记录，使路由链路可测。
3. adapter 管理 change 完成后切换到正式安装记录读取。

## Open Questions

- 正式 adapter registry 的 allowlist、denylist、版本选择和用户级命令路径存储由 adapter 管理 change 完成；本 change 只需要定义可被替换的读取接口。
