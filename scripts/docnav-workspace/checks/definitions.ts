import { defineChecks } from "./normalization.ts";
import { PROFILE_FULL, PROFILE_REQUIRED } from "./model.ts";

const DEV_BIN_COPY_DIR = ".cache/docnav/verify/dev-bins";
const DEV_BIN_ENV_FILE = ".cache/docnav/verify/dev-bins.json";

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
        id: "change-plans",
        label: "change plans",
        command: "bun",
        args: ["run", "validate:change-plans"],
        ignoreOutput: [
          /^\$ bun scripts\/change-plans\/validate\.ts$/,
          /^Change plans check passed \(\d+ active, \d+ archived; draft=\d+, plan=\d+, implementation=\d+, shelved=\d+\)\.$/
        ]
      },
      {
        id: "docs-validators",
        label: "docs validators",
        command: "bun",
        args: ["run", "validate:docs"],
        ignoreOutput: [
          /^\$ bun scripts\/docs\/validate\.ts$/,
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
          /^Decision records check passed \(\d+ domains, \d+ decisions, \d+ active, \d+ aligned, \d+ unaligned, \d+ archived, \d+ candidates\)\.$/,
          /^Investigation report check passed \(\d+ of \d+ topics checked across \d+ categories; full index current\)\.$/,
          /^markdown links ok:/
        ]
      },
      {
        id: "test-evidence-ledger",
        label: "semantic Case ledger",
        command: "bun",
        args: ["run", "test-evidence", "--", "check", "--root", "."],
        ignoreOutput: [
          /^\$ bun scripts\/test-evidence\/index\.ts check --root \.$/,
          /^Test Case check passed: \d+ current test entities \(\d+ Cargo, \d+ Bun, \d+ smoke\); \d+ mapped by \d+ semantic Cases across \d+ topics\.$/
        ]
      },
      {
        id: "test-evidence-rule-tests",
        label: "test evidence ast-grep rule tests",
        command: "bun",
        args: ["run", "test:test-evidence-rules"],
        ignoreOutput: [
          /^\$ bun scripts\/test-evidence\/test-rules\.ts$/,
          /^ast-grep \d+\.\d+\.\d+$/,
          /^Running \d+ tests$/,
          /^-+ Case Details -+$/,
          /^PASS .+$/,
          /^test result: ok\. \d+ passed; 0 failed;$/
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
