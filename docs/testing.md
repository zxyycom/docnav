# 测试策略

本文定义 Docnav 自动化测试的层级、所有权、统一验证入口和一致性审计规则。具体测试用例编号账本、smoke case 清单、覆盖矩阵和发布包预验收分别由子文档维护：

- [测试用例编号账本](testing/cases.md)：可审计 Case ID、证明目标和源码 `@case` marker 的映射。
- [Smoke Case 清单](testing/smoke-cases.md)：JavaScript smoke 的 case inventory 和新增用例规则。
- [覆盖矩阵](testing/coverage.md)：跨入口、命令族和 capability 的最低覆盖目标。
- [发布包验证](testing/release.md)：release package 的本地预验收和 CI/CD 验证边界。

稳定字段、错误码、命令语义、adapter 行为和 schema shape 以 [文档导航](navigation.md#规则所有权) 指向的 owner 文档为准；测试文档只记录覆盖目标和验收边界。

## 测试层级

| 层级 | 核心目标 |
| --- | --- |
| schema | 原始协议、manifest、probe 和各 operation readable 输出分别通过独立 schema；readable schema 用于示例和工具输出校验，不作为完整机器协议 |
| 单元 | parser、ref、默认值、分页、错误映射和其它自定义逻辑不变量 |
| 集成 | `docnav` 配置优先级、adapter 选择、adapter 管理、invoke 单请求、输出模式和进程通道 |
| 端到端 | 真实 CLI、MCP bridge、release package、协议映射、精简输出和 continuation 链路 |

## 测试所有权

测试按“用户可观察契约”和“自定义逻辑不变量”划分所有权。同一行为只有在证明不同责任时才跨层测试：Rust tests 证明内部不变量，JavaScript smoke 证明真实进程入口的外部契约。

### JavaScript smoke

JavaScript smoke 从发布给用户的可执行入口验证外部契约。覆盖范围按以下维度评估：

- 所有命令族的代表性路径，以及关键成功和失败场景。
- `readable-view`、`readable-json` 和 `protocol-json` 三种输出模式。
- 退出码、`stdout`、`stderr` 及其相互约束。
- warning 的承载位置和 schema 校验。
- `invoke` 的 stdin 请求链路。
- 分页、continuation 和终止行为。
- core、adapter、MCP bridge 和 release package 的跨进程链路。

每个契约维度至少保留一个代表性用例。同一校验规则下的多个同类非法值视为一个等价类，只选择能证明外部行为的用例。覆盖完整性由契约维度判断，不以代码覆盖率或参数组合数量衡量。

### Rust tests

Rust tests 负责具有独立出错空间的自定义逻辑。每个用例应明确证明一个分支、状态转换、算法边界或数据不变量，例如：

- 参数 token 消费边界和兼容规则。
- operation 参数所有权。
- Markdown 解析、ref 生成和定位。
- Unicode 字符预算、分页和终止规则。
- protocol、manifest 和 probe decode pipeline 的 schema invalid、typed deserialize invalid 和 semantic invalid 边界。
- diagnostics、direct CLI argv compatibility、document output orchestration、低层 JSON writer 和 paging helper 的可观察行为边界。

以下行为由 JavaScript smoke 验证外部契约，无需在 Rust 中建立重复矩阵：

- `clap` 自带的解析行为，例如简单缺少必填参数。
- 无自定义分支的字段透传、转换和输出模式枚举选择。
- 同一校验规则下的多个等价非法值。
- `readable-view` block framing、stdout/stderr 分流和用户可观察输出边界。

### 代码组织

- Rust 白盒测试放在对应 `tests.rs` 子模块，主实现文件只声明测试模块。
- 测试通过模块可见性访问私有实现，生产 API 的可见性保持不变。
- 单个测试只证明一个自定义不变量。
- 参数解析测试保持少量高价值用例；新增用例必须覆盖新的兼容规则、token 消费边界或 operation 参数所有权不变量。
- 跨层测试必须分别断言内部不变量和外部进程契约，不重复相同的参数组合矩阵。

## 脚本与工具依赖

验证脚本和按需工具依赖的运行方式由 [工程工具链](tooling.md) 拥有。本节只定义测试验证边界：

- 不要求依赖预先全局安装；脚本应通过项目命令或按需工具执行保持可复现。
- `pnpm run typecheck:scripts` 证明脚本模块 contract 和边界类型一致，不替代真实 CLI、schema 或进程 smoke 验证。
- `pnpm run lint:scripts` 证明脚本源码没有未使用变量/函数、显式 `any` 和常见静态质量问题。

## 统一验证入口

常规交付前使用 Docnav workspace 综合验证入口：

```bash
pnpm run verify:docnav-workspace
```

该入口默认运行 full profile，是常规交付前的完整验证入口。

日常开发可先跑 required profile：

```bash
pnpm run verify:docnav-workspace:required
```

required profile 只保留快速、确定性的必需门禁，用于日常开发中缩短反馈周期，适合改文档、修脚本或调验证逻辑时先跑。full profile 在 required profile 基础上追加质量观测扫描与测试、CLI smoke、Rust 全量测试、cargo clippy 和 OpenSpec 严格校验。

质量观测在 full profile 中保护扫描链路本身：工具封装测试、扫描执行、配置读取和输出结构必须通过。Lizard、scc 和 PMD CPD 的指标值只进入快照、报告和 warning records，不作为失败条件。

required profile 包含 `typecheck:scripts` 和 `lint:scripts`，确保 `.ts` 脚本类型 contract 与静态质量规则不会在常规快速门禁之外漂移。

开发期快捷入口：

| 命令 | 用途 |
| --- | --- |
| `pnpm run verify:docnav-workspace:required` | 快速门禁，只跑必需检查 |
| `pnpm run verify:docnav-workspace:full` | 完整门禁，显式运行 full profile |
| `pnpm run smoke:docnav` | 对当前开发构建运行 core 和 Markdown CLI smoke |
| `pnpm run cli:dev -- <args>` | 构建并运行当前开发版 `docnav` |
| `pnpm run cli:dev -- docnav-markdown <args>` | 构建并运行当前开发版 Markdown adapter |
| `pnpm --silent dnm <args>` | 运行当前开发版 Markdown adapter，只保留命令结果和失败诊断 |

局部改动仍可先运行范围更小的命令或 required profile；跨 Rust、文档、OpenSpec、schema、示例或输出层边界的交付，最终应运行 `pnpm run verify:docnav-workspace` 或 `pnpm run verify:docnav-workspace:full`。具体检查项和输出过滤规则由验证脚本维护，本节只定义 profile 用途和交付要求。

## 一致性审计

交付前检查：

1. 新增或修改测试能追溯到 [文档导航](navigation.md#规则所有权) 指向的 owner 文档。
2. 测试文档只记录覆盖目标和验收边界，不重新定义稳定字段、错误码或命令语义。
3. schema 和示例只校验 protocol/readable 输出映射，不成为新的业务语义来源。
4. OpenSpec change 只作为变更依据、验收和审计历史，不作为日常实现主入口。
5. 当测试暴露规范缺口时，先更新 owner 文档，再同步 schema、示例、实现和验证脚本。
6. 涉及共享 helper 的改动必须覆盖可观察外部行为：direct CLI warning placement、protocol-json stdout purity、readable warning placement、adapter direct machine command boundary、Markdown pagination mechanics 和 schema/decode/semantic invalid paths。
