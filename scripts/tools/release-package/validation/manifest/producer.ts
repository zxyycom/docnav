import type { ReleaseProducer } from "../../config.ts";
import { isRecord } from "../../../foundation/src/type-guards.ts";
import {
  assert,
  assertPositiveInteger,
} from "../assertions.ts";

export function validateProducer(
  producer: unknown,
): asserts producer is ReleaseProducer {
  assert(isRecord(producer), "manifest.producer must be an object");
  assert(
    producer.kind === "local" || producer.kind === "github-actions",
    "manifest.producer.kind must be local or github-actions",
  );

  if (producer.kind === "local") {
    assert(producer.workflow === null, "local producer.workflow must be null");
    assert(producer.run_id === null, "local producer.run_id must be null");
    assert(
      producer.run_attempt === null,
      "local producer.run_attempt must be null",
    );
    return;
  }

  assert(
    typeof producer.workflow === "string" && producer.workflow.length > 0,
    "github-actions producer.workflow must be present",
  );
  assertPositiveInteger(producer.run_id, "github-actions producer.run_id");
  assertPositiveInteger(
    producer.run_attempt,
    "github-actions producer.run_attempt",
  );
}
