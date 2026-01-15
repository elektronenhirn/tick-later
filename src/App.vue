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
const editingTodo = ref<Todo | null>(null);

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
    showTodoForm.value = false;
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

async function handleEditTodo(todoData: { id: string; title: string; description?: string; revisitAt: string }) {
  try {
    const updatedTodo = await invoke<Todo>("update_todo", {
      id: todoData.id,
      title: todoData.title,
      description: todoData.description,
      revisitAt: new Date(todoData.revisitAt).toISOString(),
      clearDescription: !todoData.description
    });
    const index = todos.value.findIndex(t => t.id === todoData.id);
    if (index !== -1) {
      todos.value[index] = updatedTodo;
    }
    closeEditForm();
  } catch (e) {
    error.value = `Failed to update todo: ${e}`;
  }
}

function openEditForm(todo: Todo) {
  editingTodo.value = todo;
}

function closeEditForm() {
  editingTodo.value = null;
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
    <div class="paper-texture"></div>
    <div class="ruled-lines"></div>

    <header class="app-header">
      <div class="header-content">
        <div class="brand">
          <div class="logo-mark">TL</div>
          <div class="brand-text">
            <h1>Tick Later</h1>
            <p class="tagline">a personal journal for tasks</p>
          </div>
        </div>
        <div class="header-date">
          {{ new Date().toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' }) }}
        </div>
      </div>
      <div class="header-rule"></div>
    </header>

    <div v-if="error" class="error-notice">
      <span class="error-stamp">ERROR</span>
      <span class="error-text">{{ error }}</span>
      <button @click="loadTodos" class="retry-link">Try again</button>
    </div>

    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <span>Opening journal...</span>
    </div>

    <div v-else class="app-content">
      <TodoList
        :todos="todos"
        @toggle-complete="handleToggleComplete"
        @delete-todo="handleDeleteTodo"
        @add-todo="handleAddTodo"
        @update-todo="handleUpdateTodo"
        @edit-todo="openEditForm"
      />
    </div>

    <!-- Floating Compose Button -->
    <button @click="openTodoForm" class="compose-btn" title="New entry">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14"/>
      </svg>
    </button>

    <!-- Add Modal -->
    <Transition name="modal">
      <div v-if="showTodoForm" class="modal-overlay" @click="closeTodoForm">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h2>New Entry</h2>
            <button @click="closeTodoForm" class="close-btn" title="Close">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12"/>
              </svg>
            </button>
          </div>
          <TodoForm @add-todo="handleAddTodo" />
        </div>
      </div>
    </Transition>

    <!-- Edit Modal -->
    <Transition name="modal">
      <div v-if="editingTodo" class="modal-overlay" @click="closeEditForm">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h2>Edit Entry</h2>
            <button @click="closeEditForm" class="close-btn" title="Close">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12"/>
              </svg>
            </button>
          </div>
          <TodoForm :editing-todo="editingTodo" @update-todo="handleEditTodo" />
        </div>
      </div>
    </Transition>
  </main>
</template>

<style scoped>
.app {
  min-height: 100vh;
  padding: 0;
  background: var(--paper);
  position: relative;
  overflow-x: hidden;
}

.paper-texture {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 400 400' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
  opacity: 0.03;
  z-index: 0;
}

.ruled-lines {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  background-image: repeating-linear-gradient(
    transparent,
    transparent 31px,
    var(--rule-line) 31px,
    var(--rule-line) 32px
  );
  opacity: 0.4;
  z-index: 0;
}

.app-header {
  position: relative;
  z-index: 1;
  padding: 40px 48px 24px;
  margin-bottom: 32px;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 20px;
}

.logo-mark {
  width: 56px;
  height: 56px;
  background: var(--ink);
  color: var(--paper);
  font-family: var(--font-display);
  font-size: 1.5rem;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  letter-spacing: -0.02em;
  box-shadow:
    2px 2px 0 var(--ink-shadow),
    inset 0 0 0 1px rgba(255,255,255,0.1);
}

.brand-text h1 {
  margin: 0;
  font-family: var(--font-display);
  font-size: 2.25rem;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: -0.03em;
  line-height: 1.1;
}

.tagline {
  margin: 4px 0 0;
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--ink-light);
  font-style: italic;
  letter-spacing: 0.02em;
}

.header-date {
  font-family: var(--font-mono);
  font-size: 0.85rem;
  color: var(--ink-light);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  padding-top: 8px;
}

.header-rule {
  height: 3px;
  background: var(--ink);
  position: relative;
}

.header-rule::after {
  content: '';
  position: absolute;
  bottom: -6px;
  left: 0;
  right: 0;
  height: 1px;
  background: var(--ink);
  opacity: 0.3;
}

.app-content {
  position: relative;
  z-index: 1;
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 48px 120px;
}

.error-notice {
  position: relative;
  z-index: 1;
  margin: 0 48px 24px;
  padding: 16px 20px;
  background: var(--error-bg);
  border: 2px solid var(--error);
  border-left-width: 6px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.error-stamp {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  font-weight: 700;
  color: var(--error);
  letter-spacing: 0.15em;
  padding: 4px 8px;
  border: 2px solid var(--error);
  transform: rotate(-2deg);
}

.error-text {
  flex: 1;
  color: var(--ink);
  font-size: 0.95rem;
}

.retry-link {
  background: none;
  border: none;
  color: var(--ink);
  font-family: var(--font-body);
  font-size: 0.9rem;
  text-decoration: underline;
  text-underline-offset: 3px;
  cursor: pointer;
  padding: 0;
}

.retry-link:hover {
  color: var(--accent);
}

.loading-state {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 100px 20px;
  gap: 20px;
  color: var(--ink-light);
  font-family: var(--font-body);
  font-style: italic;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--rule-line);
  border-top-color: var(--ink);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.compose-btn {
  position: fixed;
  bottom: 32px;
  right: 32px;
  width: 60px;
  height: 60px;
  background: var(--ink);
  border: none;
  color: var(--paper);
  cursor: pointer;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow:
    3px 3px 0 var(--ink-shadow),
    0 8px 24px rgba(0,0,0,0.15);
}

.compose-btn:hover {
  transform: translate(-2px, -2px);
  box-shadow:
    5px 5px 0 var(--ink-shadow),
    0 12px 32px rgba(0,0,0,0.2);
}

.compose-btn:active {
  transform: translate(1px, 1px);
  box-shadow:
    1px 1px 0 var(--ink-shadow),
    0 4px 12px rgba(0,0,0,0.15);
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 20px;
}

.modal-content {
  background: var(--paper);
  box-shadow:
    8px 8px 0 var(--ink-shadow),
    0 24px 80px rgba(0,0,0,0.3);
  max-width: 520px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  position: relative;
}

.modal-content::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 6px;
  background: var(--ink);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 28px 28px 0;
  margin-bottom: 8px;
}

.modal-header h2 {
  margin: 0;
  font-family: var(--font-display);
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: -0.02em;
}

.close-btn {
  background: none;
  border: 2px solid transparent;
  color: var(--ink-light);
  cursor: pointer;
  padding: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.close-btn:hover {
  color: var(--ink);
  border-color: var(--ink);
}

/* Modal Transitions */
.modal-enter-active {
  transition: opacity 0.2s ease;
}
.modal-enter-active .modal-content {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.2s ease;
}
.modal-leave-active {
  transition: opacity 0.15s ease;
}
.modal-leave-active .modal-content {
  transition: transform 0.15s ease, opacity 0.15s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .modal-content {
  opacity: 0;
  transform: translateY(-20px) scale(0.98);
}
.modal-leave-to .modal-content {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}

@media (max-width: 768px) {
  .app-header {
    padding: 24px 24px 16px;
  }

  .header-content {
    flex-direction: column;
    gap: 12px;
  }

  .logo-mark {
    width: 48px;
    height: 48px;
    font-size: 1.25rem;
  }

  .brand-text h1 {
    font-size: 1.75rem;
  }

  .app-content {
    padding: 0 24px 100px;
  }

  .compose-btn {
    bottom: 24px;
    right: 24px;
    width: 52px;
    height: 52px;
  }

  .modal-overlay {
    padding: 16px;
  }

  .modal-header {
    padding: 24px 24px 0;
  }
}
</style>

<style>
@import url('https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,400;0,9..144,700;1,9..144,400&family=Source+Sans+3:ital,wght@0,400;0,500;0,600;1,400&family=JetBrains+Mono:wght@400;500&display=swap');

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  /* Typography */
  --font-display: 'Fraunces', Georgia, serif;
  --font-body: 'Source Sans 3', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-mono: 'JetBrains Mono', 'SF Mono', Consolas, monospace;

  /* Colors - Light (Warm Cream Paper) */
  --paper: #faf8f3;
  --paper-alt: #f5f2ea;
  --ink: #1a1614;
  --ink-light: #6b6560;
  --ink-shadow: rgba(26, 22, 20, 0.25);
  --rule-line: #d4cfc5;

  --accent: #8b4513;
  --accent-light: #a0522d;

  --urgent: #b91c1c;
  --urgent-bg: #fef2f2;
  --soon: #c2410c;
  --soon-bg: #fff7ed;
  --tomorrow: #1d4ed8;
  --tomorrow-bg: #eff6ff;
  --future: #047857;
  --future-bg: #ecfdf5;

  --success: #15803d;
  --success-bg: #f0fdf4;
  --error: #b91c1c;
  --error-bg: #fef2f2;

  font-family: var(--font-body);
  font-size: 16px;
  line-height: 1.6;
  font-weight: 400;
  color: var(--ink);
  background-color: var(--paper);

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  min-width: 320px;
  min-height: 100vh;
}

#app {
  width: 100%;
  margin: 0;
  padding: 0;
}

/* Scrollbar styling */
::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

::-webkit-scrollbar-track {
  background: var(--paper-alt);
}

::-webkit-scrollbar-thumb {
  background: var(--rule-line);
  border: 2px solid var(--paper-alt);
}

::-webkit-scrollbar-thumb:hover {
  background: var(--ink-light);
}

/* Selection */
::selection {
  background: var(--ink);
  color: var(--paper);
}

@media (prefers-color-scheme: dark) {
  :root {
    --paper: #1c1917;
    --paper-alt: #292524;
    --ink: #fafaf9;
    --ink-light: #a8a29e;
    --ink-shadow: rgba(0, 0, 0, 0.5);
    --rule-line: #44403c;

    --accent: #d97706;
    --accent-light: #f59e0b;

    --urgent: #ef4444;
    --urgent-bg: #2c1810;
    --soon: #f97316;
    --soon-bg: #2c1d10;
    --tomorrow: #3b82f6;
    --tomorrow-bg: #172032;
    --future: #10b981;
    --future-bg: #0f2520;

    --success: #22c55e;
    --success-bg: #14261a;
    --error: #ef4444;
    --error-bg: #2c1810;
  }

  ::-webkit-scrollbar-track {
    background: var(--paper);
  }

  ::-webkit-scrollbar-thumb {
    background: var(--rule-line);
    border-color: var(--paper);
  }
}
</style>
