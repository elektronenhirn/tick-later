export interface Todo {
  id: string;
  title: string;
  description?: string;
  revisit_at: string; // ISO date string
  completed: boolean;
  created_at: string; // ISO date string
}