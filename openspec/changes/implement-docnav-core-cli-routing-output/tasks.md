一句话核心：实现 `docnav` 核心 CLI 的路由、adapter 选择和输出层，让用户命令稳定调用 adapter。

## 0. 审计门禁

- [x] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现。

执行说明：0.1 完成前只进行审阅和文案修正；0.1 完成后按 1.x-4.x 执行实现与验证。

## 1. CLI 参数与配置

- [ ] 1.1 新增 `crates/docnav` workspace member，提供 package/bin 名称均为 `docnav` 的核心 CLI 可执行入口。
  验收：workspace 可构建 `docnav` package，`docnav` 可解析顶层命令。
- [ ] 1.2 建立核心 CLI 命令结构，覆盖 `outline/read/find/info/init/doctor/version/config`。
  验收：每个命令都有确定的参数入口和稳定错误路径。
- [ ] 1.3 实现 core 通用参数解析：path、ref、query、page、limit_chars、output 和 adapter。
  验收：page 省略时解析为 `1`，limit_chars 为有限正整数，已知 flag 的下一个 token 固定作为值。
- [ ] 1.4 实现兼容性参数处理。
  验收：未知 flag、多余 positional 和当前 operation 不使用的已知 flag 生成包含 `ignored_tokens`、kind 和 reason 的 warning 后忽略；未知 flag 不吞后续 token；已知有值 flag 消费紧跟 token；已知必需参数缺失、已知 flag 缺少值或值非法返回 `INVALID_REQUEST`。
- [ ] 1.5 实现项目配置、用户配置和内置默认值读取，支持 `defaults.adapter`、`defaults.limit_chars` 和 `defaults.output`。
  验收：显式参数优先于项目配置，项目配置优先于用户配置，用户配置优先于内置默认值；`defaults.adapter` 只在未传 `--adapter` 时参与预选。
- [ ] 1.6 实现 `config get|set|unset|list` 的 MVP 行为。
  验收：`set/unset` 默认写项目配置，传 `--user` 写用户配置；`list` 输出所有支持 key 的当前生效值和来源；未知 key 返回 `INVALID_REQUEST`。
- [ ] 1.7 实现 `config list --path <path> [--operation outline|read|find|info]`。
  验收：命令按文档命令规则解析 path、选择 adapter，并输出该文档上下文下的选中 adapter、最终 core 默认参数和来源。
- [ ] 1.8 实现基础管理命令。
  验收：`init` 幂等创建 `.docnav/docnav.json`，`version` 输出 crate version，`doctor` 输出 `checks` 数组并在失败项存在时非零退出。

## 2. Path 与 Adapter 选择

- [ ] 2.1 实现项目根发现和 path 规范化。
  验收：相对 path 基于启动 cwd 解析，项目根内 path 可传相对路径，项目根外可访问文件可传规范化绝对路径，规范路径使用 `/`。
- [ ] 2.2 实现临时 adapter registry 读取接口。
  验收：读取 `<project-root>/.docnav/adapters.json`，接受 `{ "version": 1, "adapters": [{ "id": "...", "command": "relative/path" }] }`，按 `adapters` 数组顺序遍历。
- [ ] 2.3 校验临时 registry 记录。
  验收：`command` 只接受相对项目根路径；registry 缺失表示无显式候选；JSON 损坏、字段缺失或非法命令路径返回 `INVALID_REQUEST`。
- [ ] 2.4 实现 adapter 预选。
  验收：`--adapter` 优先，其次项目/用户 `defaults.adapter`，最后基于候选 manifest `formats[].extensions[]` 等轻量信息推断；无法推断时预选为空。
- [ ] 2.5 实现 manifest/probe 当前契约校验。
  验收：manifest/probe 输出符合当前 schema 和语义时才可选中；有效 `supported: false` 记录 `PROBE_UNSUPPORTED` 证据并继续遍历；预选 adapter 的 schema 或语义校验失败记录候选证据并继续 registry 遍历；registry 遍历候选的 schema 或语义校验失败直接返回 adapter/protocol 错误。
- [ ] 2.6 实现统一 registry 遍历函数。
  验收：函数接收已尝试 adapter id 集合，跳过已尝试项，返回第一个 probe 成功的 adapter；所有候选失败时返回 `FORMAT_UNKNOWN`。
- [ ] 2.7 实现候选证据输出。
  验收：`FORMAT_UNKNOWN` details 包含稳定 JSON 数组，每项包含 `adapter_id`、`stage`、`code`、`reason` 和 `details`；至少覆盖 `ADAPTER_NOT_FOUND` 和 `PROBE_UNSUPPORTED`。

## 3. Invoke 与输出映射

- [ ] 3.1 启动 adapter manifest、probe 和 invoke 子进程。
  验收：adapter 子进程 cwd 设置为项目根；没有项目根时使用启动 cwd。
- [ ] 3.2 构造 invoke 请求。
  验收：请求显式写入最终 page、limit_chars、ref、query 等 core 通用参数；core 不从 manifest 读取默认参数，不合成格式 options。
- [ ] 3.3 实现 protocol-json 输出。
  验收：成功和 core 自身错误都输出完整 protocol envelope；core 错误无法确定 operation 时使用 `operation: null`。
- [ ] 3.4 实现默认 text 和 readable-json 输出。
  验收：输出阅读层结果，不包含 protocol envelope；readable-json read 保留 `content_type`。
- [ ] 3.5 实现 warning 承载。
  验收：text 在正常阅读文本后拼接 warning；readable-json 输出包含 `warnings` 数组；protocol-json stdout 保持 schema-valid envelope 且不增加 `warnings` 字段，warning 写入 stderr；warning 不改变可成功执行命令的退出码。
- [ ] 3.6 实现错误 code、details、guidance 和退出码映射。
  验收：输入错误、文档/ref/格式错误、protocol/adapter 进程错误和内部错误映射到稳定 code 与主规范退出码。
- [ ] 3.7 实现选中 adapter invoke 校验失败路径。
  验收：invoke stdout 非单一 JSON、operation/request_id 不匹配、字段缺失、类型不符、result shape 不匹配或 schema/语义校验失败时直接返回 protocol 或 adapter 进程错误。

## 4. 验证与审计

- [ ] 4.1 新增核心 CLI smoke 脚本。
  验收：使用真实 `docnav` 二进制和真实 adapter 子进程覆盖 `docnav outline -> ref -> read`。
- [ ] 4.2 将核心 CLI smoke 接入 workspace 验证入口。
  验收：workspace 验证能运行核心 CLI smoke，并在失败时返回非零。
- [ ] 4.3 覆盖文档操作输出矩阵。
  验收：outline/read/find/info 的 text、readable-json 和 protocol-json 均被验证，protocol-json 与 readable-json 业务语义一致但包装不同。
- [ ] 4.4 覆盖 adapter 选择矩阵。
  验收：覆盖 `--adapter` 预选、配置预选、core 推断预选、有效 probe 不支持后继续遍历、预选契约不一致后继续遍历、候选证据、registry 遍历契约校验失败直接返回和全失败。
- [ ] 4.5 覆盖配置、管理命令和兼容性参数处理。
  验收：覆盖项目配置、`--user` 用户配置、`config list`、`config list --path`、`init`、`version`、`doctor`、未知参数 warning 和多余参数 warning。
- [ ] 4.6 覆盖 registry 与契约校验失败场景。
  验收：覆盖临时 registry 缺失、损坏、非法相对命令路径以及 manifest/probe/invoke 当前契约校验失败。
- [ ] 4.7 完成局部审计。
  验收：用局部 diff 确认实现只修改核心 CLI、临时 registry 读取、相关测试和 workspace 验证入口。
