export interface GitInfo {
  branch: string;
  uncommitted: number;
  unpushed: number;
  last_commit_msg: string;
  last_commit_relative: string;
  has_remote: boolean;
}

export interface ProjectDetails {
  name: string;
  path: string;
  status: string;
  stack: string[];
  last_modified: string | null;
  total_size_bytes: number;
  dep_size_bytes: number;
  git: GitInfo | null;
  readme_preview: string | null;
  is_monorepo: boolean;
  ci_cd: string | null;
  runtime_version: string | null;
}

export interface FuzzyMatch {
  name: string;
  path: string;
  status: string;
  score: number;
}
