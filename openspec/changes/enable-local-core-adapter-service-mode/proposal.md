本 change 只起草默认启用的本地 core 与 adapter service mode 想法和审计入口；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Why

当前 adapter `invoke` 是每次 stdin/stdout JSON 请求后退出，启动成本会成为高频 outline/read/find/info 的性能热点。service mode 需要作为 core 与适配层的通用 fast path 一起支持，否则架构收益不完整。

## What Changes

- 新增本地 core service 与 adapter service mode 的草案；两层作为一个整体 change 处理。
- service mode 默认启用。
- 连接失败类问题 fallback 到现有 `adapter invoke` 路径，并产生内部 fast-path diagnostic 或 owner-scoped status；document success output 保持 documented payload contract。
- handshake、wire hash、frame 或内部 payload mismatch hard fail，不 fallback。
- 内部协议仅限同版本/同 build 本地 fast path，不进入 public `docnav-protocol` schema。
- 现有 adapter `invoke` 入口保留为兼容和 fallback 路径。

## Capabilities

### New Capabilities

- `local-service-mode`: 本地 core service、adapter service、内部 IPC fast path、fallback 和 mismatch failure 边界。

### Modified Capabilities

当前草案不直接修改已归档主 spec；审计门禁会确认是否需要拆到 `core-cli`、`adapter-protocol` 或新主 spec。

## Impact

- 未来会影响 `docnav` core execution path、adapter SDK serve loop、adapter lifecycle、local IPC、diagnostics、benchmark 和 integration tests。
- 可能引入 async runtime、local socket/named pipe、length-delimited framing 和 internal binary payload 依赖。
- 不改变 public protocol-json/readable success payload、primary failure projection、adapter-owned ref、pagination semantics 或 existing single-request invoke contract。
