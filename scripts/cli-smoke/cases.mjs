import {
  expectExit,
  expectProtocolSuccess,
  expectStderrEmpty,
  parseJson
} from "./assertions.mjs";

export function createCliSmokeCases({ runCli, validateSchema }) {
  async function runSuccessfulJsonCase(name, args, options) {
    const {
      schema,
      commandOptions,
      check = () => {},
      checkStderr = expectStderrEmpty
    } = options;
    const record = await runCli(name, args, commandOptions);
    expectExit(record, 0);
    checkStderr(record);
    const json = parseJson(record);
    validateSchema(record, schema, json);
    check(record, json);
    return { record, json };
  }

  async function runProtocolResponseCase(name, args, options) {
    const {
      operation,
      schema = "protocolResponse",
      check,
      ...jsonOptions
    } = options;
    return runSuccessfulJsonCase(name, args, {
      ...jsonOptions,
      schema,
      check: (record, json) => {
        expectProtocolSuccess(record, json, operation);
        check?.(record, json);
      }
    });
  }

  return { runProtocolResponseCase, runSuccessfulJsonCase };
}
