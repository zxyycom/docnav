# configure-pagination-defaults

统一 core `docnav` 和 adapter SDK direct CLI 的 pagination 默认配置。

本 change 只探索并约束 `defaults.pagination.enabled`、`defaults.pagination.limit`、`--pagination enabled|disabled` 和 `--limit <n>` 的默认值行为；具体协议字段迁移等待 `explore-structured-protocol-fields` 收敛。
