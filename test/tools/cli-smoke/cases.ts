import {
  expectExit,
  expectProtocolSuccess,
  expectStderrEmpty,
  parseJson
} from "./assertions.ts";
import type { JsonRecord } from "./assertions.ts";
import type { CommandRecord, SmokeCommandOptions } from "../smoke-harness.ts";

type JsonCaseCheck = (record: CommandRecord, json: JsonRecord) => void;
type StderrCheck = (record: CommandRecord) => void;

interface SuccessfulJsonCaseOptions {
  check?: JsonCaseCheck;
  checkStderr?: StderrCheck;
  commandOptions?: SmokeCommandOptions;
  schema: string;
}

interface ProtocolResponseCaseOptions {
  check?: JsonCaseCheck;
  checkStderr?: StderrCheck;
  commandOptions?: SmokeCommandOptions;
  operation: string;
  schema?: string;
}

interface CliSmokeCaseFactoryOptions {
  runCli: (name: string, args: string[], options?: SmokeCommandOptions) => Promise<CommandRecord>;
  validateSchema: (record: CommandRecord, name: string, value: unknown) => void;
}

export function createCliSmokeCases({ runCli, validateSchema }: CliSmokeCaseFactoryOptions) {
  async function runSuccessfulJsonCase(name: string, args: string[], options: SuccessfulJsonCaseOptions) {
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

  async function runProtocolResponseCase(name: string, args: string[], options: ProtocolResponseCaseOptions) {
    const schema = options.schema ?? "protocolResponse";
    return runSuccessfulJsonCase(name, args, {
      commandOptions: options.commandOptions,
      checkStderr: options.checkStderr,
      schema,
      check: (record, json) => {
        expectProtocolSuccess(record, json, options.operation);
        options.check?.(record, json);
      }
    });
  }

  return { runProtocolResponseCase, runSuccessfulJsonCase };
}
