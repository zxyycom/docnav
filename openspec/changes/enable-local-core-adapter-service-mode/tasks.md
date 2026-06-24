本 tasks 只给出本地 core 与 adapter service mode 的粗粒度推进顺序；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：确认 service mode 必须同时覆盖 core service 和 adapter service，不接受 adapter-only 或 core-only 半套实现。
- [ ] 1.2 阻塞级审计：确认默认启用、连接失败 fallback、mismatch hard fail 和 kill switch 的可观察行为。
- [ ] 1.3 阻塞级审计：确认内部协议不进入 public `docnav-protocol` schema，且现有 `adapter invoke` contract 保留。
- [ ] 1.4 阻塞级审计：确认首轮不引入 OS service install/start/stop 管理，除非另开 change。

## 2. 轮廓实现

- [ ] 2.1 审计通过后，定义 private handshake、framing 和 request/response envelope 的最小内部模型。
- [ ] 2.2 为 adapter SDK 增加 service serve loop，并让现有 operation handler 可复用。
- [ ] 2.3 为 core 增加 service client path、fallback path 和 mismatch hard-fail path。
- [ ] 2.4 增加默认启用与 kill switch 的入口。

## 3. 验证

- [ ] 3.1 添加 service success、connection fallback、handshake mismatch hard fail 和 invoke fallback 保留的 integration tests。
- [ ] 3.2 添加基础 benchmark 或 smoke，对比 spawn invoke 与 service path 的冷/热行为。
- [ ] 3.3 运行 cross-boundary Docnav workspace 验证。
