export interface GitHubSyncConfig {
  enabled: boolean;
  token: string;
  repo: string;
  file_path: string;
  poll_interval_secs: number;
}

export interface SyncReport {
  pulled: number;
  pushed: number;
  conflicts_resolved: number;
  unchanged: boolean;
}

export const DEFAULT_SYNC_CONFIG: GitHubSyncConfig = {
  enabled: false,
  token: "",
  repo: "",
  file_path: "todos.json",
  poll_interval_secs: 120,
};
