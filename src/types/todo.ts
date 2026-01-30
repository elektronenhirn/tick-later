export interface Todo {
  id: string;
  title: string;
  description?: string;
  revisit_at: string; // ISO date string
  completed: boolean;
  created_at: string; // ISO date string
  virtual_desktop?: number; // 0-indexed desktop number
  color?: string; // Custom hatching color (hex)
}

export interface DesktopInfo {
  current: number;
  total: number;
  names: string[];
}