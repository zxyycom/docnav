import { exitCodes } from "../config.mjs";
import { fixture, getNormalRef } from "../fixtures.mjs";
import { runCli } from "../runner.mjs";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectProtocolSuccess,
  expectStderrEmpty,
  expectStderrIncludes,
  expectStdoutEmpty,
  expectStdoutIncludes,
  parseJson
} from "../assertions.mjs";
import { validateSchema } from "../schemas.mjs";

const invalidValuesByType = Object.freeze({
  positiveInt: Object.freeze([
    Object.freeze({
      label: "zero",
      value: "0",
      stderr: (flag) => `${flag} must be a positive integer`
    }),
    Object.freeze({
      label: "nonnumeric",
      value: "abc",
      stderr: (flag) => `${flag} must be a positive integer`
    })
  ]),
  headingLevel: Object.freeze([
    Object.freeze({
      label: "zero",
      value: "0",
      stderr: () => "--max-heading-level must be an integer from 1 to 6"
    }),
    Object.freeze({
      label: "too high",
      value: "7",
      stderr: () => "--max-heading-level must be an integer from 1 to 6"
    }),
    Object.freeze({
      label: "nonnumeric",
      value: "abc",
      stderr: () => "--max-heading-level must be an integer from 1 to 6"
    })
  ]),
  output: Object.freeze([
    Object.freeze({
      label: "bogus",
      value: "bogus",
      stderr: () => 'invalid --output "bogus"'
    })
  ]),
  nonEmpty: Object.freeze([
    Object.freeze({
      label: "empty",
      value: "",
      stderr: (flag) => `${flag} must not be empty`
    })
  ])
});

const commandSpecs = Object.freeze({
  outline: Object.freeze({
    path: Object.freeze({
      required: true,
      missingStderr: "outline requires <path>",
      missingBeforeFlagArgs: Object.freeze(["--output", "text"])
    }),
    requiredFlags: Object.freeze([]),
    allowedFlags: Object.freeze({
      "--page": Object.freeze({ type: "positiveInt", sample: "1", coverMissingValue: true, coverInvalidValues: true }),
      "--limit-chars": Object.freeze({ type: "positiveInt", sample: "6000", coverInvalidValues: true }),
      "--max-heading-level": Object.freeze({ type: "headingLevel", sample: "3", coverMissingValue: true, coverInvalidValues: true }),
      "--output": Object.freeze({ type: "output", sample: "readable-json", coverMissingValue: true, coverInvalidValues: true })
    }),
    unsupportedFlagScenarios: Object.freeze([
      Object.freeze({
        name: "outline unsupported flag --unknown",
        appendArgs: Object.freeze(["--unknown", "value"]),
        stderr: "unknown or unsupported flag --unknown",
        compatibilityWarning: true
      }),
      Object.freeze({
        name: "outline extra positional unexpected",
        appendArgs: Object.freeze(["unexpected", "value"]),
        stderr: "unknown or unsupported flag unexpected",
        compatibilityWarning: true
      })
    ])
  }),
  read: Object.freeze({
    path: Object.freeze({
      required: true,
      missingStderr: "read requires <path>",
      missingBeforeFlagArgs: Object.freeze(["--ref", "$ref"])
    }),
    requiredFlags: Object.freeze([
      Object.freeze({ flag: "--ref", missingStderr: "read requires --ref <ref>" })
    ]),
    allowedFlags: Object.freeze({
      "--ref": Object.freeze({ type: "nonEmpty", sample: "$ref", coverMissingValue: true, coverInvalidValues: true }),
      "--page": Object.freeze({ type: "positiveInt", sample: "1", coverInvalidValues: true }),
      "--limit-chars": Object.freeze({ type: "positiveInt", sample: "6000", coverMissingValue: true, coverInvalidValues: true }),
      "--output": Object.freeze({ type: "output", sample: "readable-json" })
    }),
    unsupportedFlagScenarios: Object.freeze([
      Object.freeze({
        name: "read unsupported operation flag --max-heading-level",
        appendArgs: Object.freeze(["--max-heading-level", "3"]),
        stderr: "unknown or unsupported flag --max-heading-level",
        compatibilityWarning: true
      })
    ])
  }),
  find: Object.freeze({
    path: Object.freeze({
      required: true,
      missingStderr: "find requires <path>",
      missingBeforeFlagArgs: Object.freeze(["--query", "target"])
    }),
    requiredFlags: Object.freeze([
      Object.freeze({ flag: "--query", missingStderr: "find requires --query <text>" })
    ]),
    allowedFlags: Object.freeze({
      "--query": Object.freeze({ type: "nonEmpty", sample: "target", coverMissingValue: true, coverInvalidValues: true }),
      "--page": Object.freeze({ type: "positiveInt", sample: "1", coverInvalidValues: true }),
      "--limit-chars": Object.freeze({ type: "positiveInt", sample: "6000", coverInvalidValues: true }),
      "--max-heading-level": Object.freeze({ type: "headingLevel", sample: "3", coverInvalidValues: true }),
      "--output": Object.freeze({ type: "output", sample: "readable-json" })
    }),
    unsupportedFlagScenarios: Object.freeze([
      Object.freeze({
        name: "find unsupported operation flag --ref",
        appendArgs: Object.freeze(["--ref", "x"]),
        stderr: "unknown or unsupported flag --ref",
        compatibilityWarning: true
      })
    ])
  }),
  info: Object.freeze({
    path: Object.freeze({
      required: true,
      missingStderr: "info requires <path>",
      missingBeforeFlagArgs: Object.freeze(["--output", "text"])
    }),
    requiredFlags: Object.freeze([]),
    allowedFlags: Object.freeze({
      "--output": Object.freeze({ type: "output", sample: "readable-json" })
    }),
    unsupportedFlagScenarios: Object.freeze([
      Object.freeze({
        name: "info unsupported operation flag --page",
        appendArgs: Object.freeze(["--page", "1"]),
        stderr: "unknown or unsupported flag --page",
        compatibilityWarning: true
      })
    ])
  }),
  manifest: Object.freeze({
    path: Object.freeze({ required: false }),
    requiredFlags: Object.freeze([]),
    allowedFlags: Object.freeze({
      "--output": Object.freeze({ type: "protocolOutput", sample: "protocol-json" })
    }),
    unsupportedFlagScenarios: Object.freeze([
      Object.freeze({
        name: "manifest protocol-only --output text",
        args: Object.freeze(["manifest", "--output", "text"]),
        stderr: "only --output protocol-json is supported for this command"
      })
    ])
  }),
  probe: Object.freeze({
    path: Object.freeze({
      required: true,
      missingStderr: "probe requires <path>",
      missingBeforeFlagArgs: Object.freeze(["--output", "protocol-json"])
    }),
    requiredFlags: Object.freeze([]),
    allowedFlags: Object.freeze({
      "--output": Object.freeze({ type: "protocolOutput", sample: "protocol-json" })
    }),
    unsupportedFlagScenarios: Object.freeze([
      Object.freeze({
        name: "probe protocol-only --output text",
        args: (context) => ["probe", context.normal, "--output", "text"],
        stderr: "only --output protocol-json is supported for this command"
      })
    ])
  }),
  invoke: Object.freeze({
    path: Object.freeze({ required: false }),
    requiredFlags: Object.freeze([]),
    allowedFlags: Object.freeze({}),
    unsupportedFlagScenarios: Object.freeze([
      Object.freeze({
        name: "invoke positional unexpected",
        args: Object.freeze(["invoke", "unexpected"]),
        stderr: "invoke does not accept positional arguments"
      })
    ])
  })
});

export function testCliArgumentFailures() {
  const normal = fixture("normal.md");
  const ref = getNormalRef();
  const cases = generateCliArgumentFailureCases(normal, ref);

  for (const item of cases) {
    const record = runCli(item.name, item.args);
    expectExit(record, exitCodes.input);
    expectStdoutEmpty(record);
    expectStderrIncludes(record, item.stderr);
  }
}

export function testCliArgumentCompatibilityWarnings() {
  const normal = fixture("normal.md");
  const ref = getNormalRef();

  const rootHelp = runCli("docnav-markdown root help", ["--help"]);
  expectExit(rootHelp, exitCodes.success);
  expectStderrEmpty(rootHelp);
  expectStdoutIncludes(rootHelp, "Usage:");
  expectStdoutIncludes(rootHelp, "outline");

  const outlineHelp = runCli("docnav-markdown outline help", ["outline", "--help"]);
  expectExit(outlineHelp, exitCodes.success);
  expectStderrEmpty(outlineHelp);
  expectStdoutIncludes(outlineHelp, "--max-heading-level");
  expectStdoutIncludes(outlineHelp, "--output");

  const text = runCli("outline unknown equals flag text warning", [
    "outline",
    normal,
    "--unknown=value",
    "--output",
    "text"
  ]);
  expectExit(text, exitCodes.success);
  expectStderrEmpty(text);
  expectStdoutIncludes(text, "page:");
  expectStdoutWarning(text, ["--unknown=value"]);

  const unknownBeforePath = runCli("outline unknown before path readable-json warning", [
    "outline",
    "--future",
    normal,
    "--output",
    "readable-json"
  ]);
  expectExit(unknownBeforePath, exitCodes.success);
  expectStderrEmpty(unknownBeforePath);
  const unknownBeforePathJson = parseJson(unknownBeforePath);
  validateSchema(unknownBeforePath, "readableOutline", unknownBeforePathJson);
  expectStructuredWarning(
    unknownBeforePath,
    unknownBeforePathJson.warnings?.[0],
    ["--future"],
    "unknown flag"
  );

  const readable = runCli("outline unknown and extra readable-json warnings", [
    "outline",
    normal,
    "--future",
    "extra",
    "--output",
    "readable-json"
  ]);
  expectExit(readable, exitCodes.success);
  expectStderrEmpty(readable);
  const readableJson = parseJson(readable);
  validateSchema(readable, "readableOutline", readableJson);
  expectStructuredWarning(
    readable,
    readableJson.warnings?.[0],
    ["--future"],
    "unknown flag"
  );
  expectStructuredWarning(
    readable,
    readableJson.warnings?.[1],
    ["extra"],
    "extra positional"
  );

  const unused = runCli("read unused known flag readable-json warning", [
    "read",
    normal,
    "--ref",
    ref,
    "--max-heading-level",
    "3",
    "--output",
    "readable-json"
  ]);
  expectExit(unused, exitCodes.success);
  expectStderrEmpty(unused);
  const unusedJson = parseJson(unused);
  validateSchema(unused, "readableRead", unusedJson);
  expectStructuredWarning(
    unused,
    unusedJson.warnings?.[0],
    ["--max-heading-level", "3"],
    "unused native flag"
  );

  const unusedInvalid = runCli("info unused invalid limit readable-json warning", [
    "info",
    normal,
    "--limit-chars",
    "nope",
    "--output",
    "readable-json"
  ]);
  expectExit(unusedInvalid, exitCodes.success);
  expectStderrEmpty(unusedInvalid);
  const unusedInvalidJson = parseJson(unusedInvalid);
  validateSchema(unusedInvalid, "readableInfo", unusedInvalidJson);
  expectStructuredWarning(
    unusedInvalid,
    unusedInvalidJson.warnings?.[0],
    ["--limit-chars", "nope"],
    "unused known invalid flag"
  );

  const protocol = runCli("outline unknown flag protocol-json stderr warning", [
    "outline",
    normal,
    "--future",
    "--output",
    "protocol-json"
  ]);
  expectExit(protocol, exitCodes.success);
  expectStderrWarning(protocol, ["--future"]);
  expectNoJsonPayloadInStderr(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolSuccess(protocol, protocolJson, "outline");
  expectNoWarningsField(protocol, protocolJson, "protocol-json stdout");

  const manifest = runCli("manifest unknown flag stderr warning", [
    "manifest",
    "--future",
    "--output",
    "protocol-json"
  ]);
  expectExit(manifest, exitCodes.success);
  expectStderrWarning(manifest, ["--future"]);
  expectNoJsonPayloadInStderr(manifest);
  const manifestJson = parseJson(manifest);
  validateSchema(manifest, "manifest", manifestJson);
  expectNoWarningsField(manifest, manifestJson, "manifest stdout");

  const probe = runCli("probe unknown flag stderr warning", [
    "probe",
    normal,
    "--future",
    "--output",
    "protocol-json"
  ]);
  expectExit(probe, exitCodes.success);
  expectStderrWarning(probe, ["--future"]);
  expectNoJsonPayloadInStderr(probe);
  const probeJson = parseJson(probe);
  validateSchema(probe, "probe", probeJson);
  expectNoWarningsField(probe, probeJson, "probe stdout");

  const refLikeFlag = runCli("read ref value looks like flag", [
    "read",
    normal,
    "--ref",
    "--future-value",
    "--output",
    "text"
  ]);
  expectExit(refLikeFlag, exitCodes.documentRefFormat);
  expectStderrEmpty(refLikeFlag);
  expectStdoutIncludes(refLikeFlag, "REF_NOT_FOUND");
  expectStdoutIncludes(refLikeFlag, "ref=--future-value");
}

export function generateCliArgumentFailureCases(normal, ref) {
  const context = { normal, ref };
  const cases = [];

  for (const [command, spec] of Object.entries(commandSpecs)) {
    if (spec.path.required) {
      cases.push({
        name: `${command} missing path`,
        args: [command],
        stderr: spec.path.missingStderr
      });
      if (spec.path.missingBeforeFlagArgs) {
        cases.push({
          name: `${command} missing path before flag`,
          args: [command, ...resolveCliArgs(spec.path.missingBeforeFlagArgs, context)],
          stderr: spec.path.missingStderr
        });
      }
    }

    for (const requiredFlag of spec.requiredFlags) {
      cases.push({
        name: `${command} missing required ${requiredFlag.flag}`,
        args: cliBaseArgs(command, spec, context, { omitRequiredFlag: requiredFlag.flag }),
        stderr: requiredFlag.missingStderr
      });
    }

    for (const [flag, flagSpec] of Object.entries(spec.allowedFlags)) {
      if (flagSpec.coverMissingValue) {
        cases.push({
          name: `${command} missing value ${flag}`,
          args: [...cliBaseArgs(command, spec, context, { omitRequiredFlag: flag }), flag],
          stderr: `${flag} requires a value`
        });
      }

      if (flagSpec.coverInvalidValues) {
        const invalidValues = invalidValuesByType[flagSpec.type] ?? [];
        for (const invalid of invalidValues) {
          cases.push({
            name: cliInvalidValueCaseName(command, flag, invalid),
            args: [...cliBaseArgs(command, spec, context, { omitRequiredFlag: flag }), flag, invalid.value],
            stderr: invalid.stderr(flag)
          });
        }
      }
    }

    for (const scenario of spec.unsupportedFlagScenarios) {
      if (scenario.compatibilityWarning) {
        continue;
      }
      cases.push({
        name: scenario.name,
        args: cliScenarioArgs(command, spec, scenario, context),
        stderr: scenario.stderr
      });
    }
  }

  return cases;
}

function cliBaseArgs(command, spec, context, options = {}) {
  const args = [command];
  if (spec.path.required) {
    args.push(context.normal);
  }
  for (const requiredFlag of spec.requiredFlags) {
    if (requiredFlag.flag === options.omitRequiredFlag) {
      continue;
    }
    const flagSpec = spec.allowedFlags[requiredFlag.flag];
    args.push(requiredFlag.flag, cliSampleValue(flagSpec, context));
  }
  return args;
}

function cliScenarioArgs(command, spec, scenario, context) {
  if (typeof scenario.args === "function") {
    return scenario.args(context);
  }
  if (Array.isArray(scenario.args)) {
    return [...scenario.args];
  }
  return [...cliBaseArgs(command, spec, context), ...scenario.appendArgs];
}

function cliSampleValue(flagSpec, context) {
  if (flagSpec.sample === "$ref") {
    return context.ref;
  }
  return flagSpec.sample;
}

function resolveCliArgs(args, context) {
  return args.map((arg) => (arg === "$ref" ? context.ref : arg));
}

function cliInvalidValueCaseName(command, flag, invalid) {
  if (invalid.label === "empty") {
    return `${command} empty ${flag}`;
  }
  return `${command} invalid ${flag} ${invalid.label}`;
}

function expectStdoutWarning(record, expectedTokens) {
  expectStdoutIncludes(record, "id=cli_argv_ignored");
  expectStdoutIncludes(record, "effect=operation_continued");
  expectStdoutIncludes(record, "details=");
  for (const token of expectedTokens) {
    expectStdoutIncludes(record, JSON.stringify(token));
  }
}

function expectStderrWarning(record, expectedTokens) {
  expectStderrIncludes(record, "id=cli_argv_ignored");
  expectStderrIncludes(record, "effect=operation_continued");
  expectStderrIncludes(record, "details=");
  for (const token of expectedTokens) {
    expectStderrIncludes(record, JSON.stringify(token));
  }
}

function expectStructuredWarning(record, warning, expectedTokens, label = "CLI argv") {
  expect(record, Boolean(warning), `structured warning exists for ${label}`);
  expect(record, warning.id === "cli_argv_ignored", "CLI argv warning id matches");
  expect(record, warning.effect === "operation_continued", "CLI argv warning effect matches");
  expect(record, typeof warning.reason === "string" && warning.reason.length > 0, "CLI argv warning reason is nonempty");
  expect(record, Array.isArray(warning.details?.tokens), "CLI argv warning details.tokens is an array");
  for (const token of expectedTokens) {
    expect(record, warning.details.tokens.includes(token), `CLI argv warning mentions ${token}`);
  }
}

function expectNoWarningsField(record, value, label) {
  expect(record, !Object.hasOwn(value, "warnings"), `${label} omits warnings`);
}
