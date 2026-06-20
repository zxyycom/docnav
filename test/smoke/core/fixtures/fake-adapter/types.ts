export interface AdapterRequest extends Record<string, unknown> {
  arguments?: unknown;
  document?: unknown;
  operation?: unknown;
  request_id?: unknown;
}

export interface FakeAdapterOptions {
  command: string | null;
  commandArgs: string[];
  extensions: string[];
  id: string;
  log: string | null;
  mode: string;
}

export interface InvokeInput {
  request: unknown;
  stdin: string;
}

export interface OperationContext {
  adapterId: string;
  args: Record<string, unknown>;
  documentPath: string;
}

export type OperationResultBuilder = (context: OperationContext) => Record<string, unknown>;
