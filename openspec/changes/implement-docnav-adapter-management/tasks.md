一句话核心：实现正式 adapter 管理，让外部 adapter 的来源、manifest 当前契约校验和本地文件 fingerprint 都可审计。

## 0. 审计门禁

- [ ] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现；未完成本项前，1.x 及后续任务全部处于阻塞状态。

## 1. 安装记录与列表

- [ ] 1.1 （阻塞：等待 0.1 用户审计确认）设计并实现用户级安装 registry，按 adapter id 和 version 保存命令路径、install mode、manifest 快照、来源、SHA-256 fingerprint 和健康状态。
- [ ] 1.2 （阻塞：等待 0.1 用户审计确认）设计并实现项目级 adapter 策略 registry，支持 allowlist、denylist 和当前版本选择，且不得保存本机命令路径。
- [ ] 1.3 （阻塞：等待 0.1 用户审计确认）实现 adapter resolver，合并用户级安装 registry 与项目级策略 registry，输出可调用的候选版本集合。
- [ ] 1.4 （阻塞：等待 0.1 用户审计确认）实现 `docnav adapter list`，展示身份、版本、格式、来源、项目策略命中、install mode 和上次可用状态。
- [ ] 1.5 （阻塞：等待 0.1 用户审计确认）处理 registry 缺失、损坏、项目策略引用缺失版本和 adapter 不可用状态。
- [ ] 1.6 （阻塞：等待 0.1 用户审计确认）设计并实现用户级 managed adapter artifact 目录，确保二进制制品与配置文件目录分离。
- [ ] 1.7 （阻塞：等待 0.1 用户审计确认）实现显式健康检查入口，例如 `docnav adapter list --check` 或 `docnav doctor`，用于刷新 fingerprint 和可用状态。

## 2. Install

- [ ] 2.1 （阻塞：等待 0.1 用户审计确认）实现 `docnav adapter install <source> [--mode managed|path]` 参数解析，默认 `managed`。
- [ ] 2.2 （阻塞：等待 0.1 用户审计确认）实现本地可执行文件来源解析、可执行性检查、`managed` 复制安装和 `path` 规范化绝对路径登记。
- [ ] 2.3 （阻塞：等待 0.1 用户审计确认）实现 `docnav adapter register <local-exe>` 作为 `install <local-exe> --mode path` 的语义别名。
- [ ] 2.4 （阻塞：等待 0.1 用户审计确认）实现 adapter 可执行文件 SHA-256 fingerprint 计算，输入为实际登记或托管的执行文件完整字节，保存为 `sha256` 小写十六进制值。
- [ ] 2.5 （阻塞：等待 0.1 用户审计确认）实现内置 adapter 下载简写解析、制品信息记录和 managed artifact 写入，并拒绝内置下载来源使用 `--mode path`。
- [ ] 2.6 （阻塞：等待 0.1 用户审计确认）安装时执行 manifest，校验 manifest 当前 schema、必需字段、字段类型和语义。
- [ ] 2.7 （阻塞：等待 0.1 用户审计确认）确保安装校验失败时不注册 adapter、不留下未记录的 managed artifact。
- [ ] 2.8 （阻塞：等待 0.1 用户审计确认）确保任意 URL、GitHub 链接和其它非内置远程来源失败，并给出改用内置简写或本地可执行文件的 guidance。

## 3. Update 与 Remove

- [ ] 3.1 （阻塞：等待 0.1 用户审计确认）实现 `docnav adapter update [adapter-id] [--version <version>]`，使用旧来源获取或重新验证候选版本。
- [ ] 3.2 （阻塞：等待 0.1 用户审计确认）确保 update 在 manifest 当前 schema、语义和 fingerprint 校验通过后才替换旧记录。
- [ ] 3.3 （阻塞：等待 0.1 用户审计确认）确保 update 不能用 id/version 不匹配的新候选覆盖旧记录。
- [ ] 3.4 （阻塞：等待 0.1 用户审计确认）确保 update 失败时保留旧版本并返回结构化错误。
- [ ] 3.5 （阻塞：等待 0.1 用户审计确认）实现 `docnav adapter remove <adapter-id> [--version <version>]`，清理用户级安装 registry 记录。
- [ ] 3.6 （阻塞：等待 0.1 用户审计确认）处理仍被项目策略 registry 引用时的 remove 失败或 guidance。
- [ ] 3.7 （阻塞：等待 0.1 用户审计确认）确保 `path` 模式 update 重新验证记录路径，内置 `managed` 模式 update 重新走内置映射，本地 `managed` 原始来源缺失时返回 guidance。
- [ ] 3.8 （阻塞：等待 0.1 用户审计确认）确保 remove 只删除 `managed` 模式下由 `docnav` 管理的 artifact，不删除 `path` 模式登记的用户文件。

## 4. 验证与审计

- [ ] 4.1 （阻塞：等待 0.1 用户审计确认）覆盖本地 exe `managed`/`path` install、register、list、update、remove 和 fingerprint 失配测试。
- [ ] 4.2 （阻塞：等待 0.1 用户审计确认）覆盖内置 source key 解析失败、任意 URL 拒绝和 manifest 校验失败测试。
- [ ] 4.3 （阻塞：等待 0.1 用户审计确认）覆盖内置下载来源拒绝 `--mode path`、下载制品进入 managed artifact 目录和校验失败清理测试。
- [ ] 4.4 （阻塞：等待 0.1 用户审计确认）覆盖项目 allowlist、denylist、version 省略匹配全版本、version 指定只匹配单版本和当前版本选择测试。
- [ ] 4.5 （阻塞：等待 0.1 用户审计确认）验证普通文档操作不重新计算 fingerprint，显式健康检查发现 fingerprint 失配后 resolver 不会调用该 adapter。
- [ ] 4.6 （阻塞：等待 0.1 用户审计确认）用局部 diff 确认只修改 adapter 管理和相关测试范围。
