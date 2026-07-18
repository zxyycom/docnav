# unify-output-with-injected-rendering

将 document operation 输出收敛为 `protocol-json` 与注入 renderer 两条路径；两条路径消费同一个 `ProtocolResponse`，core CLI 的默认阅读输出由内置 `readable-view` renderer 提供。
