本 spec delta 定义 `interactive-outline-selection` 对 `core-cli` 的新增行为要求：`docnav outline <path> --interactive` 提供面向人类的 outline 选择与 read 编排；当前 change 只在 `openspec/changes/interactive-outline-selection/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: 核心 CLI 必须支持交互式 outline 选择
`docnav` core CLI MUST support `docnav outline <path> --interactive` as a human-only workflow. Interactive outline selection MUST reuse the normal core CLI document operation pipeline to select an adapter and obtain outline entries, MUST present those entries for terminal selection, and MUST invoke existing read semantics for the selected refs without changing adapter protocol, ref format, outline entry semantics, read result semantics, or machine-readable output contracts.

#### Scenario: 交互式 outline 选择后读取选中 refs
- **WHEN** 调用方在 TTY 中执行 `docnav outline docs/guide.md --interactive`
- **AND** adapter outline 返回多个 entries
- **AND** 用户选择一个或多个 entries 并确认
- **THEN** `docnav` 使用每个选中 entry 的 ref
- **THEN** `docnav` 按用户确认的选择顺序执行既有 read 语义
- **THEN** adapter protocol request 和 response shape 不因交互模式改变

#### Scenario: 交互式 outline 不支持机器可读输出模式
- **WHEN** 调用方执行 `docnav outline docs/guide.md --interactive --output readable-json`
- **OR** 调用方执行 `docnav outline docs/guide.md --interactive --output protocol-json`
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** `docnav` 不启动交互 UI
- **THEN** stdout 不混合终端控制输出和机器可读 JSON

#### Scenario: 非 TTY 不进入交互
- **WHEN** 调用方在非 TTY stdin 或 stdout 环境中执行 `docnav outline docs/guide.md --interactive`
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** 错误说明 interactive outline requires a terminal 或等价诊断
- **THEN** `docnav` 不启动交互 UI

#### Scenario: 用户取消交互
- **WHEN** 调用方执行 `docnav outline docs/guide.md --interactive`
- **AND** 用户在选择界面取消或退出
- **THEN** `docnav` 不执行 read
- **THEN** `docnav` 以成功退出或等价的用户取消结果结束
- **THEN** adapter protocol 和 readable/protocol JSON 输出契约不新增取消字段

#### Scenario: 空 outline 不进入选择
- **WHEN** 调用方执行 `docnav outline docs/guide.md --interactive`
- **AND** adapter outline 返回空 entries
- **THEN** `docnav` 不展示多选界面
- **THEN** `docnav` 以人类可读方式说明没有可选择条目或等价结果
- **THEN** `docnav` 不执行 read
