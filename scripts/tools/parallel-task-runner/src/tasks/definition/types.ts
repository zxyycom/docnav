export type StringList = string | readonly string[] | undefined;

export interface TaskDefinition {
  id: string;
  label?: string;
  type?: string;
  mutex?: StringList;
  dependsOn?: StringList;
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
