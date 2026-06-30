/**
 * Docnav 代码质量观测配置。
 *
 * 用途：定义质量观测的扫描范围、排除规则、6 类默认 code areas、
 * 工具参数、warning policy 和产物目录默认值。配置文件是质量观测行为的 owner，
 * 脚本实现不将这些规则散落为硬编码逻辑。
 */

import { errorMessage } from "../../errors.ts";
import { isStringArray } from "../../type-guards.ts";
import type { QualityConfig } from "./schema.ts";

/**
 * 读取 JSON 编码的字符串数组环境变量。
 *
 * 仅用于少量测试/临时覆盖，不改变默认配置语义。
 */
function readJsonStringArrayEnv(name: string): string[] {
  const raw = process.env[name];
  if (!raw) return [];

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err: unknown) {
    throw new Error(`${name} must be a JSON array of strings: ${errorMessage(err)}`, { cause: err });
  }

  if (!isStringArray(parsed)) {
    throw new Error(`${name} must be a JSON array of strings`);
  }

  return parsed;
}

export const DEFAULT_CONFIG = Object.freeze({
  /** 配置版本，用于 baseline 比较时追踪配置变更 */
  version: "0.8.1",

  include: [
    "crates/**/*.rs",
    "scripts/**/*.ts",
    "test/**/*.ts"
  ],

  excludeDirs: [
    ".git",
    "target",
    "node_modules",
    ".venv",
    ".uv-cache",
    ".ruff_cache",
    "dist",
    "build",
    "__pycache__",
    ".pnpm-store",
    ".cache",
    ".tmp",
    ".log"
  ],

  generatedFiles: [
    "scripts/tools/validators/generated/**",
    "scripts/generated/**",
    "**/generated/**"
  ],

  codeAreas: Object.freeze({
    "rust-production": {
      description: "Rust production code (non-test, non-fixture, non-generated)",
      globs: ["crates/*/src/**/*.rs"],
      excludeGlobs: [
        "crates/*/src/tests/**",
        "**/tests.rs",
        "**/fixtures/**",
        "**/generated/**"
      ],
      warningPolicy: "strict"
    },
    "rust-tests": {
      description: "Rust test code",
      globs: [
        "crates/*/tests/**/*.rs",
        "crates/*/src/tests/**/*.rs",
        "**/tests.rs"
      ],
      excludeGlobs: ["**/fixtures/**", "**/generated/**"],
      warningPolicy: "relaxed"
    },
    "typescript-production-scripts": {
      description: "TypeScript production scripts",
      globs: ["scripts/**/*.ts"],
      excludeGlobs: [
        "scripts/tools/validators/**",
        "scripts/**/*.test.ts",
        "test/**",
        "**/fixtures/**",
        "**/generated/**"
      ],
      warningPolicy: "moderate"
    },
    "typescript-validation-smoke": {
      description: "TypeScript validation and smoke test scripts",
      globs: [
        "scripts/tools/validators/**/*.ts",
        "scripts/**/*.test.ts",
        "test/smoke/**/*.ts",
        "test/tools/**/*.ts",
        "test/**/*.ts"
      ],
      excludeGlobs: [
        "test/**/cases/**",
        "test/**/fixtures/**",
        "**/generated/**"
      ],
      warningPolicy: "relaxed"
    },
    "fixtures-examples": {
      description: "Test fixtures, cases, example data, and sample I/O",
      globs: [
        "**/fixtures/**",
        "**/cases/**",
        "**/examples/**",
        "docs/examples/**",
        "docs/schemas/**",
        "test/fixtures/**"
      ],
      excludeGlobs: ["**/generated/**"],
      warningPolicy: "watchlist-only"
    },
    "generated": {
      description: "Generated files and explicitly marked generated content",
      globs: ["**/generated/**"],
      excludeGlobs: [],
      warningPolicy: "exclude-warnings"
    }
  }),

  lizard: {
    /** 圈复杂度 thresholds（用于 warning，不用于阻断） */
    cyclomaticComplexity: {
      /** 绝对下限：低于此值的函数不触发 CC warning */
      absoluteFloor: 10,
      /** 仅 changed function 且 delta > 此值时触发 warning */
      changedDelta: 5
    },
    functionCodeDensity: {
      absoluteFloor: 50,
      changedDelta: 20,
      lowComplexityAllowance: {
        /** 简单函数以低圈复杂度为准；低于该值时允许更长的线性流程。 */
        maxCyclomaticComplexityExclusive: 5,
        codeLineFloor: 150
      }
    },
    parameterCount: {
      absoluteFloor: 5,
      changedDelta: 2
    }
  },

  scc: {
    fileCodeLines: {
      absoluteFloor: 300,
      changedDelta: 100,
      lowDecisionTokenAllowance: {
        maxDecisionTokens: 10,
        codeLineFloor: 500
      }
    }
  },

  pmdCpd: {
    /** CPD cache miss task 的最大并发数；任务按 code area 并行执行 */
    maxParallelTasks: 4,
    /** 按 code area 拆分的 minimum tokens（超过此阈值才报告） */
    minimumTokens: Object.freeze({
      "rust-production": 75,
      "rust-tests": 100,
      "typescript-production-scripts": 75,
      "typescript-validation-smoke": 100,
      "fixtures-examples": 150,
      "generated": 200
    }),
    /** 全局默认 minimum tokens（当 code area 未显式配置时使用） */
    defaultMinimumTokens: 100,
    /** changed scope 中重复片段数量相对 baseline 的最小增长量 */
    duplicateFragments: {
      changedDelta: 0
    }
  },

  acceptedWarnings: Object.freeze([
    {
      ruleId: "pmd-cpd-duplicate-code",
      sourceTool: "pmd-cpd",
      codeArea: "rust-production",
      metric: "duplicate-tokens",
      value: 86,
      suggestionIncludes: [
        "crates/docnav-protocol/src/envelope.rs",
        "crates/docnav-protocol/src/operation_result.rs"
      ],
      reason:
        "OperationArguments::operation and OperationResult::operation map the same Operation enum variants at separate protocol request and result boundaries. Keeping the mappings separate preserves boundary ownership; extracting shared code would couple request-argument decoding to result-envelope projection for little maintenance gain."
    },
    {
      ruleId: "pmd-cpd-duplicate-code",
      sourceTool: "pmd-cpd",
      codeArea: "rust-production",
      metric: "duplicate-tokens",
      value: 114,
      suggestionIncludes: [
        "crates/docnav-diagnostics/src/details/payload.rs",
        "crates/docnav-standard-parameters/src/resolution.rs"
      ],
      reason:
        "AdapterConfigSourceDetails is the serialized diagnostics payload, while StandardParameterConfigSourceIssue is the standard-parameter handoff fact. Keeping the constructors separate preserves crate and owner boundaries; extracting a shared type would couple fact capture to protocol projection for trivial field assignment."
    },
    {
      ruleId: "scc-file-code-lines",
      sourceTool: "scc",
      path: "crates/docnav-protocol/src/error.rs",
      codeArea: "rust-production",
      metric: "code-lines",
      value: 452,
      reason:
        "ProtocolError intentionally keeps protocol error constructors, DiagnosticRecord projection, owner/location projection, and serde rules in one owner file while the strict input contract stabilizes. The file has low decision-token density, and splitting it now would add navigation indirection without reducing behavior risk."
    }
  ]),

  report: {
    topN: 10,
    /** 人类可读报告中使用的展示时区；metrics.json 保持 ISO UTC timestamp */
    timeZone: "Asia/Shanghai",
    showWatchlist: true,
    watchlistMax: 20
  },

  artifactDir: "artifacts/docnav-quality",

  /** 工具可用性：如何发现 lizard/scc/pmd 命令 */
  tools: {
    lizard: {
      /** Python 解释器（优先使用 DOCNAV_LIZARD_CMD 环境变量，其次 'python3' 或 'python'（Windows）） */
      command: process.env.DOCNAV_LIZARD_CMD || (
        process.platform === "win32" ? "python" : "python3"
      ),
      args: ["-m", "lizard"]
    },
    scc: {
      /** scc 3.7.0 的命令名或路径；wrapper 依赖该版本的 Provider/ULOC CSV header */
      command: process.env.DOCNAV_SCC_CMD || "scc",
      args: readJsonStringArrayEnv("DOCNAV_SCC_ARGS")
    },
    pmdCpd: {
      /** PMD CPD 的命令，使用快速启动脚本或直接 Java 调用 */
      command: "pmd",
      args: ["cpd"]
    }
  }
}) satisfies QualityConfig;
