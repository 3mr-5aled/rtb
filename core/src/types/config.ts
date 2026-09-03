export interface ProjectRootEntry {
  path: string;
  label: string;
  emoji: string;
}

export interface CleanDepsConfig {
  daysInactive: number;
  targets: string[];
}

export interface GitHealthConfig {
  scanRoots: string[];
}

export interface RtbConfig {
  version: string;
  projectRoots: Record<string, ProjectRootEntry>;
  backupRoot?: string;
  configRoot?: string;
  templateDir?: string;
  cleanDeps?: CleanDepsConfig;
  staleThresholdDays?: number;
  gitHealth?: GitHealthConfig;
}
