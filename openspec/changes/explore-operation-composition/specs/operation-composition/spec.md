## 一句话核心

operation composition SHALL be explored as a core/SDK direction before Docnav commits to specific command names, fields, schemas, or adapter protocol changes.

## 文档状态

本 change 只在 `openspec/changes/explore-operation-composition/` 下形成未审核的未来计划和探索材料，不影响现有其它文档、主规范或实现任务。

## ADDED Requirements

### Requirement: operation composition 作为探索方向

Docnav exploration artifacts SHALL treat operation composition as a future direction for improving reading ergonomics. This exploration SHALL focus on composing existing `outline`, `read`, `find` and `info` behavior before proposing new public commands or protocol fields.

#### Scenario: 记录候选组合

- **WHEN** a candidate workflow repeats existing document operations
- **THEN** the exploration SHALL record the workflow, expected user benefit, likely owning layer and unresolved contract questions
- **AND** SHALL NOT treat the candidate as selected for implementation

### Requirement: composition 默认归属 core/SDK

Operation composition exploration SHALL use core CLI or reusable SDK helpers as the default ownership assumption unless a later implementation proposal proves that format-specific adapter semantics are required.

#### Scenario: 评估归属层

- **WHEN** a candidate composition can be expressed by sequencing existing operations
- **THEN** the exploration SHALL classify it as core/SDK-owned by default
- **AND** SHALL keep format adapters focused on parsing, ref generation/parsing, pagination and single operation semantics

### Requirement: public contract 延后定稿

The exploration SHALL defer concrete command names, flags, typed protocol result shape, readable-view mapping and continuation details to a later implementation change. The follow-up implementation change MUST decide whether the composition is core-owned protocol behavior or caller-owned orchestration. Core-owned composition MUST define one `ProtocolResponse` result consumed by both public document output modes.

#### Scenario: 进入实现前

- **WHEN** a candidate composition is selected for implementation
- **THEN** a follow-up implementation change SHALL define the concrete public contract, validation materials and tests
- **AND** core-owned composition SHALL declare its typed protocol result, `protocol-json` schema/example and built-in `readable-view` renderer mapping

### Requirement: 候选池与临时筛选标准

The exploration SHALL maintain a candidate pool and temporary screening criteria for operation composition ideas. The criteria SHALL support comparison during future discussion without selecting an implementation.

#### Scenario: 维护候选池

- **WHEN** a candidate pattern is added to the exploration
- **THEN** the exploration SHALL record it as a discussion candidate
- **AND** SHALL NOT assign it implementation priority or final public surface

#### Scenario: 使用临时筛选标准

- **WHEN** candidates are compared before a follow-up implementation change exists
- **THEN** the exploration SHALL use temporary criteria covering operation reuse, core/SDK ownership, reduced caller effort, opaque refs, continuation clarity, public surface reuse, raw protocol impact and spikeability
- **AND** SHALL treat those criteria as provisional rather than final acceptance rules
