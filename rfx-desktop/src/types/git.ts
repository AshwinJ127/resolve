export interface BranchInfo {
  name: string;
  current: boolean;
  ahead: number;
  behind: number;
  upstream?: string;
}

export interface RepoOverview {
  current_branch: string;
  branches: BranchInfo[];
  clean: boolean;
}
