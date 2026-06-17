import {
  expectExit,
  expectProtocolSuccess,
  expectStderrEmpty,
  parseJson
} from "./assertions.ts";

export function createCliSmokeCases({ runCli, validateSchema }: any) {
  async function runSuccessfulJsonCase(name: any, args: any, options: any) {
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

  async function runProtocolResponseCase(name: any, args: any, options: any) {
    const {
      operation,
      schema = "protocolResponse",
      check,
      ...jsonOptions
    } = options;
    return runSuccessfulJsonCase(name, args, {
      ...jsonOptions,
      schema,
      check: (record: any, json: any) => {
        expectProtocolSuccess(record, json, operation);
        check?.(record, json);
      }
    });
  }

  return { runProtocolResponseCase, runSuccessfulJsonCase };
}
