# configure-pagination-defaults

统一 core `docnav` 和 adapter SDK direct CLI 的 pagination 默认配置。

本 change 将入口侧分页默认值表达为 `defaults.pagination.enabled` 和 `defaults.pagination.limit`，并让 `--pagination enabled|disabled`、`--limit <n>` 和 adapter `invoke` request `arguments` 进入同一标准参数来源模型。Protocol shape 继续使用 `arguments.limit` 与 `arguments.page`；配置/default 可以补足缺失的已注册分页参数，但不会回写原始 protocol request，也不会新增 protocol `pagination` 字段。
