import type { ToolAvailability } from "../../../model/schema.ts";
import { checkLizard } from "./lizard.ts";
import { checkPmdCpd } from "./pmd-cpd.ts";
import { checkScc } from "./scc.ts";

export async function checkTools(rootDir: string): Promise<ToolAvailability[]> {
  return Promise.all([
    checkLizard(rootDir),
    checkScc(rootDir),
    checkPmdCpd(rootDir)
  ]);
}
