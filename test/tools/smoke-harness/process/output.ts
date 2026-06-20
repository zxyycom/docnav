import { Buffer } from "node:buffer";

interface CommandOutputCapture {
  append: (chunk: unknown, streamName: "stderr" | "stdout") => void;
  snapshot: () => { stderr: string; stdout: string };
}

export function createCommandOutputCapture(
  maxBuffer: number,
  onMaxBufferExceeded: () => void
): CommandOutputCapture {
  let stdout = "";
  let stderr = "";
  let stdoutBytes = 0;
  let stderrBytes = 0;
  let maxBufferExceeded = false;

  return {
    append(chunk, streamName) {
      const text = commandOutputText(chunk);
      const bytes = Buffer.byteLength(text, "utf8");
      if (streamName === "stdout") {
        stdout += text;
        stdoutBytes += bytes;
      } else {
        stderr += text;
        stderrBytes += bytes;
      }
      if (stdoutBytes + stderrBytes > maxBuffer && !maxBufferExceeded) {
        maxBufferExceeded = true;
        onMaxBufferExceeded();
      }
    },
    snapshot() {
      return { stdout, stderr };
    }
  };
}

function commandOutputText(chunk: unknown): string {
  if (Buffer.isBuffer(chunk)) {
    return chunk.toString("utf8");
  }
  if (typeof chunk === "string") {
    return chunk;
  }
  return String(chunk);
}
