export type TaskEnv = Record<string, string | undefined>;

export type StringList = string | readonly string[] | undefined;

export interface TaskDefinition {
  id: string;
  label?: string;
  type?: string;
  mutex?: StringList;
  dependsOn?: StringList;
  env?: TaskEnv;
  envFile?: string;
  tasks?: readonly TaskDefinition[];
  run?: (task: NormalizedTask) => unknown | Promise<unknown>;
  [key: string]: unknown;
}

export interface NormalizedTask extends TaskDefinition {
  label: string;
  type: string;
  mutex: string[];
  dependsOn: string[];
  tasks?: undefined;
}
