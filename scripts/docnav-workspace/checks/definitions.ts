import { defineChecks } from "./normalization.ts";
import { PROFILE_FULL, PROFILE_REQUIRED } from "./model.ts";

const DEV_BIN_COPY_DIR = ".cache/docnav/verify/dev-bins";
const DEV_BIN_ENV_FILE = ".cache/docnav/verify/dev-bins.json";

const testRunnerSuccessOutput = [
  /^\$ bun test(?: .*)?$/,
  /^bun test v\d+\.\d+\.\d+ \([0-9a-f]+\)$/,
  /^.*\.test\.ts:$/,
  /^\(pass\) .+ \[[\d.]+(?:ms|s)\]$/,
  /^\s*\d+ pass$/,
  /^\s*0 fail$/,
  /^\s*\d+ expect\(\) calls$/,
  /^Ran \d+ tests? across \d+ files?\. \[[\d.]+(?:ms|s)\]$/
];

const cargoProgressOutput = [
  /^\s*(Checking|Compiling) .*$/,
  /^\s*Blocking waiting for file lock on .+$/,
  /^\s*Finished `.*` profile .*$/
];

const cargoTestSuccessOutput = [
  ...cargoProgressOutput,
  /^\s*Running unittests .*$/,
  /^\s*Running tests[\\/].*$/,
  /^\s*Doc-tests .*$/,
  /^running \d+ tests?$/,
  /^test .* \.\.\. ok$/,
  /^test result: ok\..*$/
];

const qualityWarningOutput = [
  /^Quality check status: warning$/,
  /^Warnings: \d+ total \(\d+ changed, \d+ regressions\)$/,
  /^This is a quick quality check, not a full quality scan\.$/,
  /^Showing first \d+ warnings:$/,
  /^\s*\d+\. \[.+\] .+$/,
  /^\s*Accepted reason: .+$/,
  /^\s*\.\.\. and \d+ more warnings$/,
  /^Detailed report: .+$/,
  /^Warning records: .+$/
];

const qualityVerificationWarningOutput = [
  /^Quality verification status: warning$/,
  /^Warnings without accepted reason: \d+ total \(\d+ changed, \d+ regressions\)$/,
  /^Showing first \d+ warnings without accepted reason:$/,
  /^\s*\d+\. \[.+\] .+$/,
  /^\s*\.\.\. and \d+ more warnings without accepted reason$/,
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
        command: "bun",
        args: ["run", "typecheck:scripts"],
        ignoreOutput: [
          /^\$ tsgo -p tsconfig\.json$/
        ]
      },
      {
        id: "lint-scripts",
        label: "TypeScript script lint",
        command: "bun",
        args: ["run", "lint:scripts"],
        ignoreOutput: [
          /^\$ eslint --max-warnings 0 --cache --cache-location \.eslintcache --cache-strategy content$/
        ]
      },
      {
        id: "quality-quick-check",
        label: "quality quick check",
        command: "bun",
        args: [
          "scripts/quality/scan.ts",
          "--profile",
          "quick",
          "--artifact-dir",
          "artifacts/docnav-quality/quick"
        ],
        env: {
          DOCNAV_QUALITY_TIMINGS: "1"
        },
        allowOutput: [
          ...qualityWarningOutput
        ],
        warningOutput: [
          /^Quality check status: warning$/m
        ]
      },
      {
        id: "docs-validators",
        label: "docs validators",
        command: "bun",
        args: ["run", "validate:docs"],
        ignoreOutput: [
          /^\$ bun scripts\/docs\/validate\.ts$/,
          /^test case catalog ok:/,
          /^json syntax ok:/,
          /^schema strict compile ok:/,
          /^schema ok:/,
          /^protocol response operation\/result binding ok$/,
          /^protocol response error details shape ok$/,
          /^readable error details shape ok$/,
          /^protocol\/readable mapping ok:/,
          /^error details ok:/,
          /^manifest example consistency ok:/,
          /^document output mode consistency ok:/,
          /^Decision records check passed \(\d+ areas, \d+ decisions, \d+ active, \d+ archived\)\.$/,
          /^markdown links ok:/
        ]
      },
      {
        id: "workspace-verifier-script-tests",
        label: "workspace verifier script tests",
        command: "bun",
        args: ["run", "test:workspace-verifier"],
        ignoreOutput: [
          ...testRunnerSuccessOutput
        ]
      },
      {
        id: "validator-script-tests",
        label: "validator script tests",
        command: "bun",
        args: ["run", "test:validators"],
        ignoreOutput: [
          ...testRunnerSuccessOutput
        ]
      },
      {
        id: "release-package-script-tests",
        label: "release package script tests",
        command: "bun",
        args: ["run", "test:release-package-scripts"],
        ignoreOutput: [
          ...testRunnerSuccessOutput
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
        command: "bun",
        args: ["run", "quality:test"],
        ignoreOutput: [
          ...testRunnerSuccessOutput
        ]
      },
      {
        id: "quality-full-check",
        label: "quality full check",
        command: "bun",
        args: [
          "scripts/quality/scan.ts",
          "--profile",
          "full",
          "--with-baseline",
          "--verification-output"
        ],
        env: {
          DOCNAV_QUALITY_TIMINGS: "1"
        },
        dependsOn: ["quality-internal-tests"],
        allowOutput: [
          ...qualityVerificationWarningOutput
        ],
        warningOutput: [
          /^Quality verification status: warning$/m
        ]
      },
      {
        id: "docnav-development-smoke",
        label: "docnav development smoke",
        tasks: [
          {
            id: "docnav-development-binaries",
            label: "docnav development binaries",
            command: "bun",
            args: [
              "scripts/docnav-dev/build-bins.ts",
              "--quiet",
              "--output-env-json",
              DEV_BIN_ENV_FILE,
              "--copy-to",
              DEV_BIN_COPY_DIR
            ],
            mutex: ["cargo-build"],
            ignoreOutput: [
              /^dev binaries ok: DOCNAV_BIN$/
            ]
          },
          {
            id: "docnav-core-development-smoke",
            label: "docnav core development smoke",
            dependsOn: ["docnav-development-binaries"],
            envFile: DEV_BIN_ENV_FILE,
            command: "bun",
            args: ["test/docnav-core-smoke.ts"],
            ignoreOutput: [
              ...smokeSuccessOutput("Docnav Core Development Smoke", ".log/smoke/core/latest.log")
            ]
          },
          {
            id: "docnav-development-artifacts-cleanup",
            label: "docnav development artifacts cleanup",
            command: "bun",
            args: [
              "scripts/docnav-dev/build-bins.ts",
              "--cleanup",
              "--output-env-json",
              DEV_BIN_ENV_FILE,
              "--copy-to",
              DEV_BIN_COPY_DIR
            ],
            dependsOn: ["docnav-core-development-smoke"],
            ignoreOutput: [
              /^dev binary artifacts cleaned$/
            ]
          }
        ]
      },
      {
        id: "cargo-clippy",
        label: "cargo clippy",
        command: "cargo",
        args: ["clippy", "--locked", "--workspace", "--all-targets", "--", "-D", "warnings"],
        mutex: ["cargo-build"],
        ignoreOutput: [
          ...cargoProgressOutput
        ]
      },
      {
        id: "cargo-test",
        label: "cargo test",
        command: "cargo",
        args: ["test", "--locked", "--workspace"],
        mutex: ["cargo-build"],
        ignoreOutput: [
          ...cargoTestSuccessOutput
        ]
      },
      {
        id: "openspec",
        label: "openspec",
        command: "bun",
        args: ["run", "validate:openspec"],
        ignoreOutput: [
          /^\$ bun run openspec validate --all --strict$/,
          /^✓ /,
          /^Totals: \d+ passed, 0 failed .*$/,
          /^- Validating\.\.\.$/
        ]
      }
    ]
  }
]);

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
