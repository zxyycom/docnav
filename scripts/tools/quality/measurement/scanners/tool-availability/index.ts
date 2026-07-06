import type { ToolAvailability } from "../../../model/schema.ts";
import { checkJscpd } from "./jscpd.ts";
import { checkLizard } from "./lizard.ts";
import { checkScc } from "./scc.ts";

export async function checkTools(rootDir: string): Promise<ToolAvailability[]> {
  return Promise.all([
    checkLizard(rootDir),
    checkScc(rootDir),
    checkJscpd(rootDir)
  ]);
}
