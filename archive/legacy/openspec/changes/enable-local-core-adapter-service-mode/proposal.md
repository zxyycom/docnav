本 proposal 已按 `adopt-core-linked-adapter-libraries` 改写为 core service 性能问题；它不再把 local service mode 作为 adapter implementation source。

## Why

默认 adapter implementation source 已收敛为 core release 内置 adapter-layer workspace crates 和 static registry。若 service mode 继续描述 adapter service 或 invoke fallback，会重新引入独立 adapter runtime 和进程来源，和当前边界冲突。仍可保留的需求是：降低 core CLI 高频调用的启动成本、缓存 project/config/registry 状态，并保证 document success output 不变。

## What Changes

- 将本 change 限定为 core service 性能、启动成本和缓存策略探索。
- service 不提供 adapter executable discovery、adapter artifact hosting 或 adapter implementation source。
- core service 只缓存 core-owned 状态，例如 project context、配置解析结果、static registry metadata 或安全可失效的 adapter layer metadata。
- adapter layer 仍来自当前 core release static registry；service 只调用同一 adapter library handle。
- 不改变 public `protocol-json`、`readable-view`、ref、pagination 或 adapter-owned parser/navigation 语义。

## Capabilities

### New Capabilities

- `local-service-mode`: core-local service performance/cache boundary for current-release document operations.

### Modified Capabilities

无。

## Impact

- 未来可能影响 `docnav` core startup path、cache invalidation、doctor/status output 和 performance benchmarks。
- 不影响 adapter contract、adapter implementation source、schema/examples 或 release package file set。
