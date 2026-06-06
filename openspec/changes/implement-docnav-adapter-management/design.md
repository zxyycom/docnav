**一句话核心：adapter 管理维护安装记录、本地文件 fingerprint 和项目级 id/version 策略记录；运行期选择由核心 resolver 完成。**

## Context

v0 要求 `docnav adapter list/install/update/remove` 是正式能力。该 change 依赖核心 CLI 的配置/状态模型，并为核心 CLI 后续 adapter 发现提供正式 registry。

registry 分为两层：用户级安装 registry 保存本机可执行入口、来源和健康状态；项目级 adapter 策略 registry 只保存可共享的 id/version allowlist、denylist 和当前版本选择。adapter 管理命令只维护这些记录，不解析文档格式，也不参与 MCP tool 映射。

## Goals / Non-Goals

**Goals:**

- 支持内置 adapter 下载简写和本地可执行文件两类首期安装来源。
- 安装和更新时执行 manifest，校验 schema 和协议兼容性。
- 建立用户级安装 registry，按 adapter id 和 version 保存命令路径、安装模式与安装元数据。
- 建立用户级 managed adapter artifact 目录，用于保存由 `docnav` 托管的 adapter 可执行制品，且与配置文件目录分离。
- 建立项目级 adapter 策略 registry，按 id/version 管理 allowlist、denylist 和当前版本选择，且不写入本机路径或来源细节。
- adapter 可执行文件记录 SHA-256 fingerprint，并在 install/register、update 和显式健康检查中重新验证；普通文档操作不重新计算 fingerprint。
- 更新失败保留旧版本。
- remove 清理 `docnav` 管理的安装记录，并处理项目策略引用。

**Non-Goals:**

- 不实现新的 adapter 格式能力。
- 不改变 adapter invoke 协议。
- 不让 MCP 管理 adapter。
- 不支持任意 URL、GitHub 链接或其它远程地址的动态下载解析；非内置来源必须是本地可执行文件。

## Decisions

1. 用户级安装 registry 保存真实可执行入口。
   - registry 使用 adapter id 作为一级键，adapter version 作为二级键。
   - 每个版本记录保存命令路径、install mode、manifest 快照、来源、SHA-256 fingerprint 信息和健康状态。
   - list 可以不启动 adapter 也显示身份、版本与格式。
   - 健康状态需要按来源执行额外检查。

2. 项目级 adapter 策略 registry 不保存本机路径。
   - 项目 registry 只保存 adapter id/version 级策略。
   - 项目 registry 不保存命令路径、绝对路径、用户名目录、来源 URL 或 fingerprint。
   - allowlist 启用时，只允许匹配 allowlist 的 adapter 参与选择。
   - denylist 启用时，即使用户机器已安装匹配 adapter，也不得使用。
   - allowlist 和 denylist 的条目都以 adapter id 为主；未指定 version 时匹配该 id 的所有版本，指定 version 时只影响该版本。
   - 同时启用 allowlist 和 denylist 时，先用 allowlist 限定候选，再用 denylist 排除；同一 id/version 同时命中时 denylist 优先。
   - 项目 registry 可以为某个 adapter id 指定当前使用版本；未指定时由核心 resolver 在该 id 的可用版本中按确定性规则选择。

3. adapter resolver 先合并 registry，再执行健康与协议校验。
   - resolver 从用户级安装 registry 读取 id/version/command。
   - resolver 应用项目级 allowlist、denylist 和当前版本选择。
   - resolver 丢弃 registry 中已标记不可用、显式健康检查已发现 fingerprint 失配或协议不兼容的版本。
   - resolver 和普通文档操作不为 fingerprint 校验读取整个 adapter 可执行文件。
   - 如果指定当前版本但用户级 registry 中不存在或不可用，返回结构化不可用错误，而不是静默选择其它版本。

4. SHA-256 fingerprint 是低频审计边界。
   - fingerprint 算法固定为 `sha256`，输入为实际登记或托管的 adapter 可执行文件完整字节，保存值使用小写十六进制。
   - install、register、update 和显式健康检查必须重新计算 fingerprint。
   - 默认 list 可以展示 registry 中的上次健康状态；刷新 fingerprint 需要显式健康检查，例如 `docnav adapter list --check` 或 `docnav doctor`。
   - 普通 `outline/read/find/info` 不执行 fingerprint 重新计算。
   - fingerprint 只用于检测已登记 adapter 文件是否发生静默变化，不证明来源可信、作者可信或内容安全。
   - 显式健康检查或 update 发现 fingerprint 不一致时，必须将记录标记为不可用或返回 `ADAPTER_UNAVAILABLE`，并在 guidance 中说明需要重新安装或更新记录。

5. install mode 区分托管制品和路径登记。
   - `managed` 表示 `docnav` 将候选 adapter 可执行文件复制或原子移动到用户级 managed adapter artifact 目录，并在 registry 中记录托管后的命令路径。
   - `path` 表示 `docnav` 不复制候选文件，只在 registry 中记录规范化绝对路径。
   - `docnav adapter install <source>` 默认使用 `managed`。
   - 内置下载简写必须使用 `managed`，不得使用 `path`。
   - 本地可执行文件来源可以使用 `managed` 或 `path`；`docnav adapter register <local-exe>` 等价于 `docnav adapter install <local-exe> --mode path`。
   - `managed` 的本地可执行文件安装复制源文件，不删除或移动用户原始文件；内置下载来源可以从临时下载位置原子移动到 managed artifact 目录。
   - managed artifact 目录不是配置文件目录；配置和 registry 只保存元数据，不嵌入二进制制品。

6. 内置下载来源必须通过简写解析到具体可执行制品。
   - `docnav` 维护内置 adapter 下载目录或静态映射；调用方使用简写，例如 `docnav adapter install markdown`。
   - 安装记录保留内置 source key 和解析后的制品信息。
   - 下载得到的可执行制品在 manifest/schema/protocol 校验通过后进入 managed artifact 目录。
   - 若 source 不是内置简写且不是本地可执行文件，必须失败且不得注册。
   - 任意 URL、GitHub 链接和其它远程地址不属于 v0 支持来源。
   - 若内置 source 无法解析、下载或执行 manifest，则不得注册。

7. update 使用旧记录来源并采用先验证后替换。
   - 新候选 manifest/schema/protocol/fingerprint 全部通过后才替换旧记录。
   - 失败时旧版本保持可用，并返回结构化错误。
   - 若新候选 manifest 的 adapter id 或 version 与目标记录不匹配，不得覆盖其它 id/version 记录。
   - `path` 模式 update 重新验证当前记录中的路径并刷新 manifest 快照与 fingerprint。
   - `managed` 模式 update 使用记录的来源重新获取候选；内置下载来源重新走内置映射，本地 managed 安装若原始来源路径不可用，update 失败并给出重新安装 guidance。

8. remove 必须检查项目策略引用。
   - 若目标 id 或 id/version 仍被 allowlist、denylist 或当前版本选择引用，必须失败或提供明确 guidance。
   - `managed` 模式 remove 清理 `docnav` 管理的 artifact 文件和 registry 记录。
   - `path` 模式 remove 只清理 registry 记录，不删除用户路径指向的外部文件。

## Risks / Trade-offs

- [内置下载目录需要维护] → v0 只支持内置 source key，失败时明确 guidance；任意 URL 不进入来源解析。
- [fingerprint 检查增加运行成本] → fingerprint 只在 install/register、update 和显式健康检查中重新计算，普通文档操作不执行该检查。
- [managed 与 path 语义混淆] → CLI 文案和 list 输出必须显示 install mode，register 仅作为 path 模式别名。
- [安装记录损坏] → list 和 doctor 需要返回结构化不可用状态，而不是 panic。
- [项目策略与用户安装状态不一致] → resolver 返回结构化不可用或未知 adapter 错误，并在 guidance 中区分“项目允许/选择了 id”与“本机未安装/不可用”。
- [allowlist、denylist 和当前版本选择互相叠加] → 测试覆盖未指定 version、指定 version、allow/deny 冲突和当前版本不可用。

## Migration Plan

1. 先实现用户级安装 registry、项目级 adapter 策略 registry 和 list。
2. 实现 managed artifact 目录与本地 exe `managed`/`path` install、register、update、remove。
3. 实现内置 adapter 下载简写解析和 managed 安装。
4. 将核心 CLI adapter 发现切到正式安装记录。

## Open Questions

- 内置 adapter 下载简写和静态映射需要实现时固定；错误 guidance 必须说明当前支持的 source key。
- 当前版本自动选择的确定性规则需要实现时固定，例如优先最高兼容健康版本；一旦固定，必须写入 CLI 主规范和测试。
