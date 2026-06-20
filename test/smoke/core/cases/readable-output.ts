import type { SmokeProject } from "../fixtures.ts";
import { runSuccessfulJsonCase } from "../harness.ts";
import {
  expect,
  expectNoProtocolEnvelope,
  expectString
} from "../assertions.ts";
import type { JsonRecord } from "../assertions.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";

type ReadableJsonCheck = (record: CommandRecord, json: JsonRecord) => void;

interface ReadableJsonCaseResult {
  json: JsonRecord;
  record: CommandRecord;
}

interface ReadableReadRefHandoffExpectation {
  contentIncludes: string;
  contentIncludesSummary: string;
  contentSummary: string;
  contentType?: {
    summary: string;
    value: string;
  };
  refSummary: string;
}

export function runReadableJsonCase(
  project: SmokeProject,
  name: string,
  args: string[],
  schema: string,
  check?: ReadableJsonCheck
): Promise<ReadableJsonCaseResult> {
  return runSuccessfulJsonCase(name, args, {
    commandOptions: { project },
    schema,
    check: (record, json) => {
      expectNoProtocolEnvelope(record, json);
      check?.(record, json);
    }
  });
}

export function assertReadableReadRefHandoff(
  project: SmokeProject,
  name: string,
  documentPath: string,
  ref: string,
  expectation: ReadableReadRefHandoffExpectation
): Promise<ReadableJsonCaseResult> {
  return runReadableJsonCase(project, name, [
    "read",
    documentPath,
    "--ref",
    ref,
    "--output",
    "readable-json"
  ], "readableRead", (record, json) => {
    const content = expectString(record, json.content, expectation.contentSummary);
    expect(record, json.ref === ref, expectation.refSummary);
    expect(
      record,
      content.includes(expectation.contentIncludes),
      expectation.contentIncludesSummary
    );
    if (expectation.contentType) {
      expect(
        record,
        json.content_type === expectation.contentType.value,
        expectation.contentType.summary
      );
    }
  });
}
