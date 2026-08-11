## MODIFIED Requirements

### Requirement: Markdown adapter 必须声明完整 v0 能力
`docnav-markdown` MUST 输出符合当前 manifest schema 的 manifest，并声明 Markdown 格式身份、扩展名、content type，以及 `outline`、`read`、`find`、`info` 全部 capability。Manifest 字段集合 MUST 排除协议范围字段和 `recommended_parameters`。

#### Scenario: 读取 manifest
- **WHEN** 调用方执行 `docnav-markdown manifest --output protocol-json`
- **THEN** 输出通过 manifest schema
- **THEN** capabilities 包含 `outline`、`read`、`find` 和 `info`
- **THEN** manifest 字段集合不包含 `protocol.min` 或 `protocol.max`
- **THEN** manifest 字段集合不包含 `recommended_parameters`
