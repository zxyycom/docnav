import type { FakeAdapterOptions, InvokeInput } from "./types.ts";

type OptionHandler = (options: FakeAdapterOptions, value: string) => void;

const optionHandlers: Record<string, OptionHandler> = {
  "--id": (parsed, value) => {
    parsed.id = value;
  },
  "--mode": (parsed, value) => {
    parsed.mode = value;
  },
  "--log": (parsed, value) => {
    parsed.log = value;
  },
  "--extensions": (parsed, value) => {
    parsed.extensions = value.split(",").filter(Boolean);
  }
};

export function parseOptions(args: string[]): FakeAdapterOptions {
  const parsed: FakeAdapterOptions = {
    id: "fake-adapter",
    mode: "valid",
    log: null,
    extensions: [".md", ".core"],
    command: null,
    commandArgs: []
  };
  let index = 0;
  while (index < args.length) {
    const token = args[index];
    const handler = optionHandlers[token];
    if (handler === undefined) {
      parsed.command = token ?? null;
      parsed.commandArgs = args.slice(index + 1);
      break;
    }

    handler(parsed, args[index + 1] ?? "");
    index += 2;
  }
  return parsed;
}

export function emptyInvokeInput(): InvokeInput {
  return {
    request: null,
    stdin: ""
  };
}

export async function readInvokeInput(): Promise<InvokeInput> {
  const stdin = await readStdin();
  return {
    request: parseRequestJson(stdin),
    stdin
  };
}

function parseRequestJson(stdin: string): unknown {
  if (stdin.trim().length === 0) {
    return null;
  }
  try {
    return JSON.parse(stdin) as unknown;
  } catch {
    return null;
  }
}

async function readStdin() {
  let content = "";
  for await (const chunk of process.stdin) {
    content += chunk;
  }
  return content;
}
