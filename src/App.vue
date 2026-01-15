<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Todo } from "./types/todo";
import TodoList from "./components/TodoList.vue";
import TodoForm from "./components/TodoForm.vue";

const todos = ref<Todo[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const showTodoForm = ref(false);

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
    showTodoForm.value = false; // Close modal after adding todo
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

async function handleUpdateTodo(todoData: { id: string; revisitAt: string }) {
  try {
    await invoke("update_todo", { 
      id: todoData.id, 
      revisitAt: new Date(todoData.revisitAt).toISOString() 
    });
    const todo = todos.value.find(t => t.id === todoData.id);
    if (todo) {
      todo.revisit_at = todoData.revisitAt;
    }
  } catch (e) {
    error.value = `Failed to update todo: ${e}`;
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

function openTodoForm() {
  showTodoForm.value = true;
}

function closeTodoForm() {
  showTodoForm.value = false;
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
      <TodoList 
        :todos="todos" 
        @toggle-complete="handleToggleComplete"
        @delete-todo="handleDeleteTodo"
        @add-todo="handleAddTodo"
        @update-todo="handleUpdateTodo"
      />
    </div>

    <!-- Floating Add Button -->
    <button @click="openTodoForm" class="floating-add-btn" title="Add new todo">
      <span class="plus-icon">+</span>
    </button>

    <!-- Modal Dialog for Todo Form -->
    <div v-if="showTodoForm" class="modal-overlay" @click="closeTodoForm">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>Add New Todo</h2>
          <button @click="closeTodoForm" class="close-btn" title="Close">×</button>
        </div>
        <TodoForm @add-todo="handleAddTodo" />
      </div>
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

.floating-add-btn {
  position: fixed;
  bottom: 32px;
  right: 32px;
  width: 64px;
  height: 64px;
  background: linear-gradient(135deg, #4f46e5, #7c3aed);
  border: none;
  border-radius: 50%;
  box-shadow: 0 8px 24px rgba(79, 70, 229, 0.3);
  cursor: pointer;
  transition: all 0.3s ease;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.floating-add-btn:hover {
  transform: scale(1.1);
  box-shadow: 0 12px 32px rgba(79, 70, 229, 0.4);
}

.floating-add-btn:active {
  transform: scale(0.95);
}

.plus-icon {
  color: white;
  font-size: 2rem;
  font-weight: 300;
  line-height: 1;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 20px;
}

.modal-content {
  background: white;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 600px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  animation: modalFadeIn 0.2s ease-out;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 24px 0 24px;
  margin-bottom: 16px;
}

.modal-header h2 {
  margin: 0;
  color: #1f2937;
  font-size: 1.5rem;
}

.close-btn {
  background: none;
  border: none;
  font-size: 2rem;
  color: #6b7280;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #f3f4f6;
  color: #374151;
}

@keyframes modalFadeIn {
  from {
    opacity: 0;
    transform: scale(0.9) translateY(-20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

@media (max-width: 768px) {
  .floating-add-btn {
    bottom: 24px;
    right: 24px;
    width: 56px;
    height: 56px;
  }
  
  .plus-icon {
    font-size: 1.75rem;
  }
  
  .modal-overlay {
    padding: 16px;
  }
  
  .modal-header {
    padding: 20px 20px 0 20px;
  }
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
  
  .modal-overlay {
    background: rgba(0, 0, 0, 0.7);
  }
  
  .modal-content {
    background: #1f2937;
  }
  
  .modal-header h2 {
    color: #f9fafb;
  }
  
  .close-btn {
    color: #9ca3af;
  }
  
  .close-btn:hover {
    background: #374151;
    color: #d1d5db;
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