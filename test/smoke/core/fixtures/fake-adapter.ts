import { emptyInvokeInput, parseOptions, readInvokeInput } from "./fake-adapter/input.ts";
import { recordCall } from "./fake-adapter/log.ts";
import { writeInvoke } from "./fake-adapter/output/invoke.ts";
import { writeManifest } from "./fake-adapter/output/manifest.ts";
import { writeProbe } from "./fake-adapter/output/probe.ts";
import type { FakeAdapterOptions } from "./fake-adapter/types.ts";

const options = parseOptions(process.argv.slice(2));
const invokeInput = options.command === "invoke" ? await readInvokeInput() : emptyInvokeInput();
recordCall(options, { stdin: invokeInput.request ?? invokeInput.stdin });
runCommand(options, invokeInput.request);

function runCommand(options: FakeAdapterOptions, request: unknown) {
  switch (options.command) {
    case "manifest":
      writeManifest(options);
      return;
    case "probe":
      writeProbe(options, options.commandArgs[0] ?? "");
      return;
    case "invoke":
      writeInvoke(options, request);
      return;
    default:
      console.error(`Unknown command ${options.command ?? "(missing)"}`);
      process.exit(2);
  }
}
