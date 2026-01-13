<script setup lang="ts">
import { computed } from "vue";
import type { Todo } from "../types/todo";
import TodoItem from "./TodoItem.vue";

const props = defineProps<{
  todos: Todo[];
}>();

const emit = defineEmits<{
  toggleComplete: [todoId: string];
  deleteTodo: [todoId: string];
}>();

const sortedTodos = computed(() => {
  return [...props.todos].sort((a, b) => {
    if (a.completed !== b.completed) {
      return a.completed ? 1 : -1;
    }
    return new Date(a.revisit_at).getTime() - new Date(b.revisit_at).getTime();
  });
});

const pendingTodos = computed(() => sortedTodos.value.filter(t => !t.completed));
const completedTodos = computed(() => sortedTodos.value.filter(t => t.completed));

function handleToggleComplete(todoId: string) {
  emit("toggleComplete", todoId);
}

function handleDeleteTodo(todoId: string) {
  emit("deleteTodo", todoId);
}
</script>

<template>
  <div class="todo-list">
    <div v-if="todos.length === 0" class="empty-state">
      <h3>No todos yet</h3>
      <p>Add your first todo above to get started!</p>
    </div>
    
    <div v-else>
      <div v-if="pendingTodos.length > 0" class="todo-section">
        <h3 class="section-title">Pending ({{ pendingTodos.length }})</h3>
        <div class="todos-grid">
          <TodoItem
            v-for="todo in pendingTodos"
            :key="todo.id"
            :todo="todo"
            @toggle-complete="handleToggleComplete"
            @delete-todo="handleDeleteTodo"
          />
        </div>
      </div>
      
      <div v-if="completedTodos.length > 0" class="todo-section">
        <h3 class="section-title">Completed ({{ completedTodos.length }})</h3>
        <div class="todos-grid">
          <TodoItem
            v-for="todo in completedTodos"
            :key="todo.id"
            :todo="todo"
            @toggle-complete="handleToggleComplete"
            @delete-todo="handleDeleteTodo"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.todo-list {
  min-height: 400px;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #6b7280;
}

.empty-state h3 {
  margin: 0 0 8px 0;
  font-size: 1.25rem;
}

.empty-state p {
  margin: 0;
  font-size: 1rem;
}

.todo-section {
  margin-bottom: 32px;
}

.section-title {
  margin: 0 0 16px 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: #374151;
  border-bottom: 2px solid #e5e7eb;
  padding-bottom: 8px;
}

.todos-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

@media (max-width: 768px) {
  .todos-grid {
    grid-template-columns: 1fr;
  }
}

@media (prefers-color-scheme: dark) {
  .empty-state {
    color: #9ca3af;
  }
  
  .section-title {
    color: #f3f4f6;
    border-bottom-color: #4b5563;
  }
}
</style>