<script setup lang="ts">
import { computed } from "vue";
import type { Todo } from "../types/todo";

const props = defineProps<{
  todo: Todo;
}>();

const emit = defineEmits<{
  toggleComplete: [todoId: string];
  deleteTodo: [todoId: string];
}>();

const revisitDate = computed(() => {
  return new Date(props.todo.revisit_at).toLocaleString();
});

const isOverdue = computed(() => {
  return new Date(props.todo.revisit_at) < new Date() && !props.todo.completed;
});

const createdDate = computed(() => {
  return new Date(props.todo.created_at).toLocaleDateString();
});

function handleToggleComplete() {
  emit("toggleComplete", props.todo.id);
}

function handleDelete() {
  if (confirm("Are you sure you want to delete this todo?")) {
    emit("deleteTodo", props.todo.id);
  }
}

function handleDragStart(event: DragEvent) {
  if (event.dataTransfer) {
    event.dataTransfer.setData("text/plain", props.todo.title);
    event.dataTransfer.setData("application/json", JSON.stringify(props.todo));
  }
}
</script>

<template>
  <div 
    class="todo-item"
    :class="{
      'completed': todo.completed,
      'overdue': isOverdue
    }"
    draggable="true"
    @dragstart="handleDragStart"
  >
    <div class="todo-header">
      <div class="todo-checkbox">
        <input
          type="checkbox"
          :checked="todo.completed"
          @change="handleToggleComplete"
          :id="`todo-${todo.id}`"
        />
        <label :for="`todo-${todo.id}`" class="checkbox-label"></label>
      </div>
      <button @click="handleDelete" class="delete-btn" title="Delete todo">
        ✕
      </button>
    </div>
    
    <div class="todo-content">
      <h4 class="todo-title">{{ todo.title }}</h4>
      
      <p v-if="todo.description" class="todo-description">
        {{ todo.description }}
      </p>
      
      <div class="todo-meta">
        <div class="revisit-info">
          <span class="label">Re-visit:</span>
          <span class="date" :class="{ 'overdue': isOverdue }">
            {{ revisitDate }}
          </span>
        </div>
        
        <div class="created-info">
          <span class="label">Created:</span>
          <span class="date">{{ createdDate }}</span>
        </div>
      </div>
    </div>
    
    <div class="drag-handle" title="Drag to reorder or export">
      ⋮⋮
    </div>
  </div>
</template>

<style scoped>
.todo-item {
  background: white;
  border: 2px solid #e5e7eb;
  border-radius: 12px;
  padding: 20px;
  transition: all 0.2s ease;
  cursor: grab;
  position: relative;
}

.todo-item:hover {
  border-color: #d1d5db;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.todo-item:active {
  cursor: grabbing;
}

.todo-item.completed {
  opacity: 0.7;
  border-color: #10b981;
  background: #f0fdf4;
}

.todo-item.overdue {
  border-color: #ef4444;
  border-left-width: 6px;
}

.todo-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.todo-checkbox {
  position: relative;
}

.todo-checkbox input[type="checkbox"] {
  opacity: 0;
  position: absolute;
}

.checkbox-label {
  display: block;
  width: 24px;
  height: 24px;
  border: 2px solid #d1d5db;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.checkbox-label::after {
  content: "✓";
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: white;
  font-weight: bold;
  font-size: 14px;
  opacity: 0;
  transition: opacity 0.2s;
}

.todo-checkbox input:checked + .checkbox-label {
  background: #10b981;
  border-color: #10b981;
}

.todo-checkbox input:checked + .checkbox-label::after {
  opacity: 1;
}

.delete-btn {
  background: none;
  border: none;
  color: #ef4444;
  font-size: 1.2rem;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.delete-btn:hover {
  background: #fee2e2;
}

.todo-content {
  margin-bottom: 12px;
}

.todo-title {
  margin: 0 0 8px 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: #1f2937;
  line-height: 1.4;
}

.completed .todo-title {
  text-decoration: line-through;
  color: #6b7280;
}

.todo-description {
  margin: 0 0 12px 0;
  color: #6b7280;
  line-height: 1.5;
  font-size: 0.95rem;
}

.todo-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 0.875rem;
}

.revisit-info,
.created-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.label {
  font-weight: 500;
  color: #6b7280;
  min-width: 60px;
}

.date {
  color: #374151;
}

.date.overdue {
  color: #ef4444;
  font-weight: 600;
}

.drag-handle {
  position: absolute;
  top: 8px;
  right: 8px;
  color: #9ca3af;
  font-size: 1.2rem;
  cursor: grab;
  padding: 4px;
  border-radius: 4px;
  transition: color 0.2s;
}

.drag-handle:hover {
  color: #6b7280;
}

.todo-item:active .drag-handle {
  cursor: grabbing;
}

@media (prefers-color-scheme: dark) {
  .todo-item {
    background: #1f2937;
    border-color: #4b5563;
  }
  
  .todo-item:hover {
    border-color: #6b7280;
  }
  
  .todo-item.completed {
    background: #064e3b;
    border-color: #10b981;
  }
  
  .checkbox-label {
    border-color: #6b7280;
  }
  
  .delete-btn:hover {
    background: #7f1d1d;
  }
  
  .todo-title {
    color: #f9fafb;
  }
  
  .completed .todo-title {
    color: #9ca3af;
  }
  
  .todo-description {
    color: #d1d5db;
  }
  
  .label {
    color: #9ca3af;
  }
  
  .date {
    color: #e5e7eb;
  }
  
  .drag-handle {
    color: #6b7280;
  }
  
  .drag-handle:hover {
    color: #9ca3af;
  }
}
</style>