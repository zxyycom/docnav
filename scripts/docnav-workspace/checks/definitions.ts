import { defineChecks } from "./normalization.ts";
import { PROFILE_FULL, PROFILE_REQUIRED } from "./model.ts";
import type { CheckDefinition } from "./model.ts";

const DEV_BIN_ENV_FILE = ".log/verify-docnav-workspace/dev-bins.json";

const nodeTestSuccessOutput = [
  /^TAP version \d+$/,
  /^\s*▶ /,
  /^\s*✔ /,
  /^\s*ℹ /,
  /^# Subtest:/,
  /^ok \d+ -/,
  /^1\.\.\d+$/,
  /^# (tests|suites|pass|fail|cancelled|skipped|todo|duration_ms) /
];

const cargoProgressOutput = [/^\s*(Checking|Compiling) .*$/, /^\s*Finished `.*` profile .*$/];

const qualityWarningOutput = [
  /^Quality check status: warning$/,
  /^Warnings: \d+ total \(\d+ changed, \d+ regressions\)$/,
  /^This is a quick quality check, not a full quality scan\.$/,
  /^Showing first \d+ warnings:$/,
  /^\s*\d+\. \[.+\] .+$/,
  /^\s*\.\.\. and \d+ more warnings$/,
  /^Detailed report: .+$/,
  /^Warning records: .+$/
];

export const checks = defineChecks([
  {
    id: "required-checks",
    type: PROFILE_REQUIRED,
    tasks: [
      {
        id: "cargo-fmt",
        label: "cargo fmt",
        command: "cargo",
        args: ["fmt", "--all", "--check"]
      },
      {
        id: "typecheck-scripts",
        label: "TypeScript script typecheck",
        command: "pnpm",
        args: ["run", "typecheck:scripts"],
        ignoreOutput: [
          /^\$ tsc -p tsconfig\.json$/
        ]
      },
      {
        id: "lint-scripts",
        label: "TypeScript script lint",
        command: "pnpm",
        args: ["run", "lint:scripts"],
        ignoreOutput: [
          /^\$ eslint --max-warnings 0 --cache --cache-location \.eslintcache --cache-strategy content eslint\.config\.ts scripts\/\*\*\/\*\.ts test\/\*\*\/\*\.ts$/
        ]
      },
      {
        id: "quality-quick-check",
        label: "quality quick check",
        command: "node",
        args: [
          "scripts/quality/scan.ts",
          "--profile",
          "quick",
          "--artifact-dir",
          "artifacts/docnav-quality/quick"
        ],
        allowOutput: [
          ...qualityWarningOutput
        ],
        warningOutput: [
          /^Quality check status: warning$/m
        ]
      },
      {
        id: "generated-error-rules",
        label: "generated error rules",
        command: "node",
        args: ["scripts/generate-error-rules.ts", "--check"],
        ignoreOutput: [
          /^generated error rules ok$/
        ]
      },
      {
        id: "docs-validators",
        label: "docs validators",
        tasks: docsValidatorChecks()
      },
      {
        id: "workspace-verifier-script-tests",
        label: "workspace verifier script tests",
        tasks: nodeTestFileChecks([
          ["workspace-verifier-tests", "workspace verifier tests", "scripts/docnav-workspace/verify.test.ts"],
          ["smoke-harness-tests", "smoke harness tests", "test/tools/smoke-harness.test.ts"],
          ["parallel-task-runner-tests", "parallel task runner tests", "scripts/tools/parallel-task-runner/index.test.ts"]
        ])
      },
      {
        id: "validator-script-tests",
        label: "validator script tests",
        tasks: nodeTestFileChecks([
          ["case-catalog-validator-tests", "case catalog validator tests", "scripts/tools/validators/case-catalog/index.test.ts"]
        ])
      },
      {
        id: "release-package-script-tests",
        label: "release package script tests",
        command: "node",
        args: ["--test", "scripts/tools/release-package/args.test.ts"],
        ignoreOutput: [
          ...nodeTestSuccessOutput
        ]
      },
      {
        id: "git-diff-whitespace",
        label: "git diff whitespace",
        command: "git",
        args: ["diff", "--check"],
        ignoreOutput: [
          /\b(CRLF|LF) will be replaced by (CRLF|LF)\b/i
        ]
      }
    ]
  },
  {
    id: "full-checks",
    type: PROFILE_FULL,
    tasks: [
      {
        id: "quality-internal-tests",
        label: "quality internal tests",
        tasks: nodeTestFileChecks([
          ["quality-internal-node-tests", "quality internal node tests", "scripts/tools/quality/**/*.test.ts"]
        ])
      },
      {
        id: "quality-full-check",
        label: "quality full check",
        command: "node",
        args: ["scripts/quality/scan.ts", "--profile", "full", "--with-baseline"],
        dependsOn: ["quality-internal-tests"],
        allowOutput: [
          ...qualityWarningOutput
        ],
        warningOutput: [
          /^Quality check status: warning$/m
        ]
      },
      {
        id: "docnav-development-smoke",
        label: "docnav development smoke",
        tasks: [
          {
            id: "docnav-development-binaries",
            label: "docnav development binaries",
            command: "node",
            args: ["scripts/docnav-dev/build-bins.ts", "--quiet", "--output-env-json", DEV_BIN_ENV_FILE],
            mutex: ["cargo-build"],
            ignoreOutput: [
              /^dev binaries ok: DOCNAV_BIN, DOCNAV_MARKDOWN_BIN$/
            ]
          },
          {
            id: "docnav-development-smoke-execution",
            dependsOn: ["docnav-development-binaries"],
            envFile: DEV_BIN_ENV_FILE,
            tasks: [
              {
                id: "docnav-markdown-development-smoke",
                label: "docnav-markdown development smoke",
                command: "node",
                args: ["test/docnav-markdown-smoke.ts"],
                ignoreOutput: [
                  ...smokeSuccessOutput("Docnav Markdown Development Smoke", ".log/docnav-markdown-cli-smoke/latest.log")
                ]
              },
              {
                id: "docnav-core-development-smoke",
                label: "docnav core development smoke",
                command: "node",
                args: ["test/docnav-core-smoke.ts"],
                ignoreOutput: [
                  ...smokeSuccessOutput("Docnav Core Development Smoke", ".log/docnav-core-cli-smoke/latest.log")
                ]
              }
            ]
          }
        ]
      },
      {
        id: "cargo-clippy",
        label: "cargo clippy",
        command: "cargo",
        args: ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
        mutex: ["cargo-build"],
        ignoreOutput: [
          ...cargoProgressOutput
        ]
      },
      {
        id: "cargo-test",
        label: "cargo test",
        command: "cargo",
        args: ["test", "--workspace"],
        mutex: ["cargo-build"],
        ignoreOutput: [
          ...cargoProgressOutput,
          /^\s*Running unittests .*$/,
          /^\s*Running tests[\\/].*$/,
          /^\s*Doc-tests .*$/,
          /^running \d+ tests$/,
          /^test .* \.\.\. ok$/,
          /^test result: ok\..*$/
        ]
      },
      {
        id: "openspec",
        label: "openspec",
        command: "openspec",
        args: ["validate", "--all", "--strict"],
        ignoreOutput: [
          /^✓ /,
          /^Totals: \d+ passed, 0 failed .*$/,
          /^- Validating\.\.\.$/
        ]
      }
    ]
  }
]);

function docsValidatorChecks(): CheckDefinition[] {
  return [
    docsValidatorCheck("docs-case-catalog-validator", "docs case catalog validator", "cases", [
      /^test case catalog ok:/
    ]),
    docsValidatorCheck("docs-json-validator", "docs json validator", "json", [
      /^json syntax ok:/
    ]),
    docsValidatorCheck("docs-schema-validator", "docs schema validator", "schema", [
      /^schema strict compile ok:/,
      /^schema ok:/,
      /^protocol response operation\/result binding ok$/,
      /^protocol response error details requirements ok$/
    ]),
    docsValidatorCheck("docs-mcp-validator", "docs mcp validator", "mcp", [
      /^mcp structuredContent ok:/
    ]),
    docsValidatorCheck(
      "docs-example-consistency-validator",
      "docs example consistency validator",
      "examples",
      [
        /^protocol\/readable mapping ok:/,
        /^error details ok:/,
        /^manifest example consistency ok:/,
        /^document output mode consistency ok:/
      ]
    ),
    docsValidatorCheck("docs-links-validator", "docs links validator", "links", [
      /^markdown links ok:/
    ])
  ];
}

function docsValidatorCheck(
  id: string,
  label: string,
  target: string,
  successOutput: readonly RegExp[]
): CheckDefinition {
  return {
    id,
    label,
    command: "pnpm",
    args: ["run", "validate:docs", target],
    ignoreOutput: [
      new RegExp(`^\\$ node scripts\\/docs\\/validate\\.ts "?${target}"?$`),
      ...successOutput
    ]
  };
}

function nodeTestFileChecks(testFiles: readonly [id: string, label: string, filePath: string][]): CheckDefinition[] {
  return testFiles.map(([id, label, filePath]) => ({
    id,
    label,
    command: "node",
    args: ["--test", filePath],
    ignoreOutput: [
      ...nodeTestSuccessOutput
    ]
  }));
}

function smokeSuccessOutput(title: string, logPath: string): RegExp[] {
  return [
    new RegExp(`^${escapeRegex(title)}$`),
    /^Status: passed$/,
    /^Commands: \d+$/,
    /^Log:$/,
    new RegExp(`^\\s+- ${escapeRegex(logPath)}$`)
  ];
}

function escapeRegex(value: string): string {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
