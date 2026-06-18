import {
  expectExit,
  expectProtocolSuccess,
  expectStderrEmpty,
  parseJson
} from "./assertions.ts";

export function createCliSmokeCases({ runCli, validateSchema }: ExternalValue) {
  async function runSuccessfulJsonCase(name: ExternalValue, args: ExternalValue, options: ExternalValue) {
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

  async function runProtocolResponseCase(name: ExternalValue, args: ExternalValue, options: ExternalValue) {
    const {
      operation,
      schema = "protocolResponse",
      check,
      ...jsonOptions
    } = options;
    return runSuccessfulJsonCase(name, args, {
      ...jsonOptions,
      schema,
      check: (record: ExternalValue, json: ExternalValue) => {
        expectProtocolSuccess(record, json, operation);
        check?.(record, json);
      }
    });
  }

  return { runProtocolResponseCase, runSuccessfulJsonCase };
}
