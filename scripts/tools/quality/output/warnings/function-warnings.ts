import type { FunctionMetric } from "../../model/schema.ts";
import { metricAreaWarningPolicy } from "./area-policy.ts";
import { functionKey } from "./baseline-context.ts";
import { buildMetricWarning, deltaFrom } from "./metric-warning.ts";
import type { AreaWarningPolicy, WarningCandidate, WarningContext } from "./warning-model.ts";

type FunctionWarningInput = {
  areaPolicy: AreaWarningPolicy;
  baselineFunc: FunctionMetric | undefined;
  context: WarningContext;
  func: FunctionMetric;
};

type FunctionWarningBuilder = (input: FunctionWarningInput) => WarningCandidate | null;

const FUNCTION_WARNING_BUILDERS: FunctionWarningBuilder[] = [
  buildFunctionComplexityWarning,
  buildFunctionLineWarning,
  buildFunctionParameterWarning
];

export function generateFunctionWarnings(functions: FunctionMetric[], context: WarningContext): WarningCandidate[] {
  const warnings: WarningCandidate[] = [];

  for (const func of functions) {
    const areaPolicy = metricAreaWarningPolicy(context.config, func.codeArea);
    if (!areaPolicy) continue;

    const baselineFunc = context.baselineFunctions.get(functionKey(func));
    const warningInput = { areaPolicy, baselineFunc, context, func };
    for (const buildWarning of FUNCTION_WARNING_BUILDERS) {
      const warning = buildWarning(warningInput);
      if (warning) warnings.push(warning);
    }
  }

  return warnings;
}

function buildFunctionComplexityWarning(input: FunctionWarningInput): WarningCandidate | null {
  const { areaPolicy, baselineFunc, context, func } = input;
  const ccFloor = context.config.lizard?.cyclomaticComplexity?.absoluteFloor ?? 10;
  const ccDelta = context.config.lizard?.cyclomaticComplexity?.changedDelta ?? 5;
  const baselineCc = baselineFunc?.cyclomaticComplexity?.value ?? (context.hasBaselineFunctions ? 0 : null);
  const functionComplexity = func.cyclomaticComplexity.value;
  const ccDeltaValue = deltaFrom(functionComplexity, baselineCc);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineCc,
    codeArea: func.codeArea,
    deltaFloor: ccDelta,
    deltaValue: ccDeltaValue,
    floor: ccFloor,
    isChanged: func.isChanged,
    line: func.startLine,
    message: `Function "${func.name}" in ${func.file}:${func.startLine} has cyclomatic complexity ${functionComplexity} (threshold: ${ccFloor} CC)`,
    metric: "cyclomatic-complexity",
    path: func.file,
    ruleId: "lizard-cyclomatic-complexity",
    sourceTool: "lizard",
    suggestion: "Consider breaking this function into smaller, more focused functions",
    value: functionComplexity
  });
}

function buildFunctionLineWarning(input: FunctionWarningInput): WarningCandidate | null {
  const { areaPolicy, baselineFunc, context, func } = input;
  const lineFloor = context.config.lizard?.functionCodeLines?.absoluteFloor ?? 50;
  const lineDeltaCfg = context.config.lizard?.functionCodeLines?.changedDelta ?? 20;
  const baselineFunctionLines = baselineFunc?.lines ?? (context.hasBaselineFunctions ? 0 : null);
  const functionLineDelta = deltaFrom(func.lines, baselineFunctionLines);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineFunctionLines,
    codeArea: func.codeArea,
    deltaFloor: lineDeltaCfg,
    deltaValue: functionLineDelta,
    floor: lineFloor,
    isChanged: func.isChanged,
    line: func.startLine,
    message: `Function "${func.name}" in ${func.file}:${func.startLine} has ${func.lines} code lines (Lizard NLOC; threshold: ${lineFloor} code lines)`,
    metric: "function-code-lines",
    path: func.file,
    ruleId: "lizard-function-code-lines",
    sourceTool: "lizard",
    suggestion: "Consider extracting parts of this function into separate functions",
    value: func.lines
  });
}

function buildFunctionParameterWarning(input: FunctionWarningInput): WarningCandidate | null {
  const { areaPolicy, baselineFunc, context, func } = input;
  const paramFloor = context.config.lizard?.parameterCount?.absoluteFloor ?? 5;
  const paramDeltaCfg = context.config.lizard?.parameterCount?.changedDelta ?? 2;
  const baselineParameterCount = baselineFunc?.parameterCount ?? (context.hasBaselineFunctions ? 0 : null);
  const paramDeltaValue = deltaFrom(func.parameterCount, baselineParameterCount);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineParameterCount,
    codeArea: func.codeArea,
    deltaFloor: paramDeltaCfg,
    deltaValue: paramDeltaValue,
    floor: paramFloor,
    isChanged: func.isChanged,
    line: func.startLine,
    message: `Function "${func.name}" in ${func.file}:${func.startLine} has ${func.parameterCount} parameters (threshold: ${paramFloor} parameters)`,
    metric: "parameter-count",
    path: func.file,
    ruleId: "lizard-parameter-count",
    sourceTool: "lizard",
    suggestion: "Consider using a parameter object or splitting the function",
    value: func.parameterCount
  });
}
