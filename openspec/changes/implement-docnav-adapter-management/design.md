**一句话核心：adapter 管理负责安装记录和来源可信度，不负责格式解析或 MCP 映射。**

## Context

v0 要求 `docnav adapter list/install/update/remove` 是正式能力。该 change 依赖核心 CLI 的配置/状态模型，并为核心 CLI 后续 adapter 发现提供正式安装记录。

## Goals / Non-Goals

**Goals:**

- 支持 GitHub 链接和本地可执行文件两类首期安装来源。
- 安装和更新时执行 manifest，校验 schema 和协议兼容性。
- 本地可执行文件记录 SHA-256 hash，并在运行前健康检查和 update 中重新验证。
- 更新失败保留旧版本。
- remove 清理 `docnav` 管理的安装记录，并处理项目配置引用。

**Non-Goals:**

- 不实现新的 adapter 格式能力。
- 不改变 adapter invoke 协议。
- 不让 MCP 管理 adapter。

## Decisions

1. 安装记录保存 manifest 快照、来源和可执行入口。
   - list 可以不启动 adapter 也显示身份与格式。
   - 健康状态需要按来源执行额外检查。

2. 本地可执行文件来源以 hash 作为安全边界。
   - install、list 健康检查、update 和运行前检查都必须重新计算 hash。
   - hash 不一致时返回 `ADAPTER_UNAVAILABLE`，不得静默调用。

3. GitHub 来源必须解析到具体可执行制品。
   - 安装记录保留原始 URL 和解析后的制品信息。
   - 若无法解析、下载或执行 manifest，则不得注册。

4. update 使用旧记录来源并采用先验证后替换。
   - 新候选 manifest/schema/protocol/hash 全部通过后才替换旧记录。
   - 失败时旧版本保持可用，并返回结构化错误。

5. remove 必须检查项目配置引用。
   - 若仍被显式引用，必须失败或提供明确 guidance。
   - 不删除不属于 `docnav` 管理的外部文件。

## Risks / Trade-offs

- [GitHub 链接形态不统一] → 首期只接受能解析到 adapter 发布制品的 URL，失败时明确 guidance。
- [hash 检查增加运行成本] → 仅本地 exe 来源必须检查，且可通过缓存状态优化但不能跳过安全判断。
- [安装记录损坏] → list 和 doctor 需要返回结构化不可用状态，而不是 panic。

## Migration Plan

1. 先实现安装记录模型和 list。
2. 实现本地 exe install/update/remove。
3. 实现 GitHub source 解析。
4. 将核心 CLI adapter 发现切到正式安装记录。

## Open Questions

- GitHub 发布制品的命名约定可在实现时收窄，但必须在错误 guidance 中说明当前支持形态。
