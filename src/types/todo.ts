export interface WorkspaceApp {
  command: string; // Command to run (e.g., "code .")
  working_dir?: string; // Working directory to start the app from
}

export interface Todo {
  id: string;
  title: string;
  description?: string;
  revisit_at: string; // ISO date string
  completed: boolean;
  created_at: string; // ISO date string
  virtual_desktop?: number; // 0-indexed desktop number
  color?: string; // Custom hatching color (hex)
  workspace_apps?: WorkspaceApp[]; // Applications to launch for this workspace
}

export interface DesktopInfo {
  current: number;
  total: number;
  names: string[];
}