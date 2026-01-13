<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Todo } from "./types/todo";
import TodoList from "./components/TodoList.vue";
import TodoForm from "./components/TodoForm.vue";

const todos = ref<Todo[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

async function loadTodos() {
  try {
    loading.value = true;
    error.value = null;
    todos.value = await invoke<Todo[]>("load_todos");
  } catch (e) {
    error.value = `Failed to load todos: ${e}`;
  } finally {
    loading.value = false;
  }
}

async function handleAddTodo(todoData: { title: string; description?: string; revisitAt: string }) {
  try {
    const newTodo = await invoke<Todo>("save_todo", {
      title: todoData.title,
      description: todoData.description,
      revisitAt: new Date(todoData.revisitAt).toISOString()
    });
    todos.value.push(newTodo);
  } catch (e) {
    error.value = `Failed to add todo: ${e}`;
  }
}

async function handleToggleComplete(todoId: string) {
  try {
    await invoke<boolean>("toggle_todo_completion", { id: todoId });
    const todo = todos.value.find(t => t.id === todoId);
    if (todo) {
      todo.completed = !todo.completed;
    }
  } catch (e) {
    error.value = `Failed to toggle todo: ${e}`;
  }
}

async function handleDeleteTodo(todoId: string) {
  try {
    await invoke("delete_todo", { id: todoId });
    todos.value = todos.value.filter(t => t.id !== todoId);
  } catch (e) {
    error.value = `Failed to delete todo: ${e}`;
  }
}

onMounted(() => {
  loadTodos();
});
</script>

<template>
  <main class="app">
    <header class="app-header">
      <h1>Tick Later</h1>
      <p>Personal Todo Organizer</p>
    </header>

    <div v-if="error" class="error">
      {{ error }}
      <button @click="loadTodos" class="retry-btn">Retry</button>
    </div>

    <div v-if="loading" class="loading">Loading todos...</div>

    <div v-else class="app-content">
      <TodoForm @add-todo="handleAddTodo" />
      <TodoList 
        :todos="todos" 
        @toggle-complete="handleToggleComplete"
        @delete-todo="handleDeleteTodo" 
      />
    </div>
  </main>
</template>

<style scoped>
.app {
  min-height: 100vh;
  padding: 20px;
  background: #f8fafc;
}

.app-header {
  text-align: center;
  margin-bottom: 32px;
  padding: 20px 0;
}

.app-header h1 {
  margin: 0 0 8px 0;
  font-size: 2.5rem;
  font-weight: 700;
  background: linear-gradient(135deg, #4f46e5, #7c3aed);
  background-clip: text;
  -webkit-background-clip: text;
  color: transparent;
}

.app-header p {
  margin: 0;
  color: #6b7280;
  font-size: 1.125rem;
}

.app-content {
  max-width: 1200px;
  margin: 0 auto;
}

.error {
  background: #fee2e2;
  color: #dc2626;
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 24px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.retry-btn {
  background: #dc2626;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 0.875rem;
  cursor: pointer;
}

.retry-btn:hover {
  background: #b91c1c;
}

.loading {
  text-align: center;
  padding: 60px 20px;
  color: #6b7280;
  font-size: 1.125rem;
}

@media (prefers-color-scheme: dark) {
  .app {
    background: #0f172a;
  }
  
  .app-header h1 {
    background: linear-gradient(135deg, #6366f1, #a855f7);
    background-clip: text;
    -webkit-background-clip: text;
  }
  
  .app-header p {
    color: #94a3b8;
  }
  
  .error {
    background: #7f1d1d;
    color: #fca5a5;
  }
  
  .loading {
    color: #94a3b8;
  }
}
</style>
<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  font-weight: 400;

  color: #1f2937;
  background-color: #f8fafc;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  display: flex;
  place-items: center;
  min-width: 320px;
  min-height: 100vh;
}

#app {
  width: 100%;
  margin: 0;
  padding: 0;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f8fafc;
    background-color: #0f172a;
  }
}
</style>