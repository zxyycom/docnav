本 design 只记录本地 service mode 的高层设计取向；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Context

用户明确要求 service mode 是 core 和适配层通用工具，要么一起支持，要么没有；并确认 service 默认启用。该 change 应记录方向，但避免在审计前锁定具体库版本、socket 命名、缓存策略或线程模型。

## Goals / Non-Goals

**Goals:**

- 提供默认启用的本地 core + adapter service fast path。
- 保留现有 `adapter invoke` single-request stdin/stdout JSON 入口。
- 连接失败 fallback 到旧 invoke，并记录内部 fast-path diagnostic 或 owner-scoped status。
- handshake、wire hash、frame 或 payload mismatch hard fail。
- 内部协议保持 private，不进入 public protocol schema。

**Non-Goals:**

- 不把 service mode 设计成跨语言、跨版本或远程网络协议。
- 不在首轮定义 OS service install/start/stop 管理。
- 不改变 readable-view、readable-json 或 protocol-json 的外部输出契约。
- 不改变 adapter-owned ref、pagination 或 format parsing semantics。

## Decisions

1. core service 与 adapter service 一起进入同一 change。
   - Rationale: 用户明确要求 service 是通用 core 和适配层工具，拆 adapter-only 会产生半套 fast path。

2. service 默认启用，但保留 kill switch 和 fallback。
   - Rationale: 默认启用实现性能目标；连接失败 fallback 保留可用性和排障空间。fallback 事件属于内部 fast-path diagnostic 或 owner-scoped status，不得污染 document success payload。

3. mismatch hard fail。
   - Rationale: handshake/wire mismatch 表示内部协议不可信，fallback 会掩盖版本或 build 不一致问题。

4. 内部协议 private。
   - Rationale: 目标是本地、高性能、同版本、不跨语言；public schema 仍由现有 protocol-json 拥有。

## Risks / Trade-offs

- [Risk] 默认 service 带来难排查的启动或连接问题 → Mitigation: kill switch、fallback status 和 doctor/diagnostic path 必须进入审计。
- [Risk] fallback 掩盖服务不可用 → Mitigation: 只对连接类失败 fallback，mismatch 类 hard fail。
- [Risk] internal protocol 过早稳定化 → Mitigation: 明确 private，同版本/build handshake 控制兼容边界。
- [Risk] 缓存失效错误影响读取正确性 → Mitigation: 缓存策略后续细化，首轮不能绕过 adapter-owned parsing/ref semantics。

## Open Questions

- kill switch 入口使用 flag、env、config，还是组合提供。
- 首轮是否只实现 foreground `docnav service run`，OS service 管理放后续。
- service diagnostics 的具体 surface 是 stderr diagnostic、doctor/context 输出还是 owner-scoped status；无论选择哪种，document success output 都保持 documented payload contract。
