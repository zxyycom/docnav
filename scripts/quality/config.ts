/**
 * Docnav 代码质量观测配置。
 *
 * 用途：定义质量观测的扫描范围、排除规则、6 类默认 code areas、
 * 工具参数、warning policy 和产物目录默认值。配置文件是质量观测行为的 owner，
 * 脚本实现不将这些规则散落为硬编码逻辑。
 */

import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { errorMessage } from "../tools/foundation/src/errors.ts";
import { isStringArray } from "../tools/foundation/src/type-guards.ts";
import type { QualityConfig } from "../tools/quality-core/src/model/schema.ts";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const JSCPD_BIN_NAME = process.platform === "win32" ? "jscpd.cmd" : "jscpd";
const DEFAULT_JSCPD_COMMAND = resolve(REPO_ROOT, "node_modules", ".bin", JSCPD_BIN_NAME);

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
  version: "0.8.3",

  include: [
    "crates/**/*.rs",
    "subrepos/cli-config-resolution/crates/**/*.rs",
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
      globs: [
        "crates/**/src/**/*.rs",
        "subrepos/cli-config-resolution/crates/**/src/**/*.rs"
      ],
      excludeGlobs: [
        "crates/**/src/tests/**",
        "subrepos/cli-config-resolution/crates/**/src/tests/**",
        "**/tests.rs",
        "**/fixtures/**",
        "**/generated/**"
      ],
      warningPolicy: "strict"
    },
    "rust-tests": {
      description: "Rust test code",
      globs: [
        "crates/**/tests/**/*.rs",
        "crates/**/src/tests/**/*.rs",
        "subrepos/cli-config-resolution/crates/**/tests/**/*.rs",
        "subrepos/cli-config-resolution/crates/**/src/tests/**/*.rs",
        "**/benches/**/*.rs",
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

  jscpd: {
    /** jscpd cache miss task 的最大并发数；任务按 code area 并行执行 */
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
    formatByCodeArea: Object.freeze({
      "rust-production": "rust",
      "rust-tests": "rust",
      "typescript-production-scripts": "typescript",
      "typescript-validation-smoke": "typescript",
      "fixtures-examples": null,
      "generated": null
    }),
    /** 全局默认 minimum tokens（当 code area 未显式配置时使用） */
    defaultMinimumTokens: 100,
    /** changed scope 中重复片段数量相对 baseline 的最小增长量 */
    duplicateFragments: {
      changedDelta: 0
    }
  },

  acceptedWarnings: Object.freeze([]),

  report: {
    title: "Docnav Code Quality Snapshot",
    nonBlockingNotice:
      "⚠️ 非阻断观测快照 — Lizard、scc 和 jscpd 指标值不作为合并阻断条件。Clippy 继续承担 Rust 阻断式 lint gate。",
    footerGeneratedBy: "Docnav Code Quality Observability",
    footerNotice:
      "⚠️ 本报告为**非阻断观测快照**。Lizard、scc 和 jscpd 指标值不作为合并阻断条件。Clippy 继续承担 Rust 阻断式 lint gate。",
    topN: 10,
    /** 人类可读报告中使用的展示时区；metrics.json 保持 ISO UTC timestamp */
    timeZone: "Asia/Shanghai",
    showWatchlist: true,
    watchlistMax: 20
  },

  artifactDir: "artifacts/docnav-quality",
  cacheDir: ".cache/docnav/quality",

  /** 工具可用性：如何发现 lizard/scc/jscpd 命令 */
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
    jscpd: {
      /** jscpd 通过当前仓库 devDependency 的本地 bin 提供，不依赖 baseline repo、全局 jscpd 或 cpd 命令。 */
      command: process.env.DOCNAV_JSCPD_CMD || DEFAULT_JSCPD_COMMAND,
      args: readJsonStringArrayEnv("DOCNAV_JSCPD_ARGS")
    }
  }
}) satisfies QualityConfig;
