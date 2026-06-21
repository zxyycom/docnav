# 工程工具链

本文定义 Docnav 仓库内开发脚本、临时工具和本地验证命令的工具链目标。它只拥有“如何运行和检查工程工具”的规则，不定义产品契约、CLI 行为、schema 字段或 smoke case 覆盖目标。

## 脚本语言与包管理

工具依赖按生态通过项目命令调用，避免依赖预先全局安装：

1. Node.js、JavaScript 和 TypeScript 工具使用 `pnpm`。
2. Python 工具使用 `uv`。
3. Rust 工具使用 Cargo workspace 命令或验证脚本封装的 Cargo 调用。

## Node.js / TypeScript 脚本

`scripts/` 和 `test/` 下的手写 Node.js 脚本以 TypeScript 源码为目标形态。脚本源码负责表达模块 contract、输入输出边界和共享状态类型；生成产物、分发产物或外部工具兼容层不拥有这些类型。

运行时目标：

1. 常规脚本由 Node.js 直接运行，例如 `node scripts/foo.ts`。
2. Node.js test runner 直接运行脚本测试，例如 `node --test path/to/foo.test.ts`。
3. 文件使用 `.ts`；只有包含 JSX 的源码使用 `.tsx`。
4. 脚本保持 erasable TypeScript：不使用需要 TypeScript 生成 JavaScript 的语法。需要枚举语义时，使用 `as const` 对象、string union 或职责内常量模块。
5. 相对 import 使用运行时真实扩展名，例如 `./config.ts`；类型专用符号使用 `import type` 或 inline `type` modifier。

类型检查目标：

1. 项目提供脚本专用 `tsconfig`。
2. 类型检查通过 `pnpm run typecheck:scripts` 执行。
3. 代码 lint 通过 `pnpm run lint:scripts` 执行，覆盖未使用变量、未使用函数、显式 `any` 和常见脚本错误。
4. 脚本 `tsconfig` 以 `noEmit`、`module: "nodenext"`、`target: "esnext"`、`strict`、`erasableSyntaxOnly`、`verbatimModuleSyntax`、`rewriteRelativeImportExtensions`、`allowImportingTsExtensions` 和 Node.js types 为基线。
5. 质量扫描、测试入口、验证脚本和文档引用覆盖 TypeScript 脚本源码。

运行时约束以 [Node.js TypeScript 文档](https://nodejs.org/docs/latest-v24.x/api/typescript.html) 为准；类型检查配置以 TypeScript 的 [`erasableSyntaxOnly`](https://www.typescriptlang.org/tsconfig/#erasableSyntaxOnly)、[`verbatimModuleSyntax`](https://www.typescriptlang.org/tsconfig/#verbatimModuleSyntax)、[`rewriteRelativeImportExtensions`](https://www.typescriptlang.org/tsconfig/#rewriteRelativeImportExtensions) 和 [`allowImportingTsExtensions`](https://www.typescriptlang.org/tsconfig/#allowImportingTsExtensions) 选项为准。

## 验证入口集成

`typecheck:scripts` 和 `lint:scripts` 是脚本模块 contract 的快速验证入口。前者证明脚本类型、模块边界和共享状态一致；后者证明脚本源码没有未使用变量/函数、显式 `any` 等静态质量问题。它们不替代真实 CLI、schema、进程 smoke、Rust tests 或 release package 验证。

required profile 包含 `typecheck:scripts` 和 `lint:scripts`。full profile 会追加更宽验证；profile 组成、质量观测边界和交付前取舍由 [测试策略](testing.md#统一验证入口) 维护。

验收标准：手写脚本可以同时由 Node.js 执行、被 `tsc --noEmit` 覆盖，并且不依赖 Node.js 运行时不会读取的 `tsconfig` 行为。
