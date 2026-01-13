<script setup lang="ts">
import { ref } from "vue";

const emit = defineEmits<{
  addTodo: [todo: { title: string; description?: string; revisitAt: string }];
}>();

const title = ref("");
const description = ref("");
const revisitAt = ref("");

function addTodo() {
  if (!title.value.trim() || !revisitAt.value) return;
  
  emit("addTodo", {
    title: title.value.trim(),
    description: description.value.trim() || undefined,
    revisitAt: revisitAt.value
  });
  
  title.value = "";
  description.value = "";
  revisitAt.value = "";
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  const text = event.dataTransfer?.getData("text/plain");
  if (text && !title.value) {
    title.value = text;
  }
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
}
</script>

<template>
  <div class="todo-form">
    <h2>Add New Todo</h2>
    <form @submit.prevent="addTodo" class="form">
      <div class="form-group">
        <label for="title">Title *</label>
        <input
          id="title"
          v-model="title"
          type="text"
          placeholder="Enter todo title..."
          required
          @drop="handleDrop"
          @dragover="handleDragOver"
          class="form-input"
        />
      </div>
      
      <div class="form-group">
        <label for="description">Description</label>
        <textarea
          id="description"
          v-model="description"
          placeholder="Optional description..."
          rows="3"
          class="form-textarea"
        ></textarea>
      </div>
      
      <div class="form-group">
        <label for="revisit-at">Re-visit at *</label>
        <input
          id="revisit-at"
          v-model="revisitAt"
          type="datetime-local"
          required
          class="form-input"
        />
      </div>
      
      <button type="submit" :disabled="!title.trim() || !revisitAt" class="submit-btn">
        Add Todo
      </button>
    </form>
    
    <div class="drop-zone">
      <p>💡 Drag text here to create a todo</p>
    </div>
  </div>
</template>

<style scoped>
.todo-form {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin-bottom: 24px;
}

.todo-form h2 {
  margin: 0 0 20px 0;
  color: #333;
  font-size: 1.5rem;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-weight: 500;
  color: #555;
  font-size: 0.9rem;
}

.form-input,
.form-textarea {
  padding: 12px;
  border: 2px solid #e1e5e9;
  border-radius: 8px;
  font-size: 1rem;
  transition: border-color 0.2s;
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: #4f46e5;
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
}

.submit-btn {
  padding: 12px 24px;
  background: #4f46e5;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.submit-btn:hover:not(:disabled) {
  background: #3b82f6;
}

.submit-btn:disabled {
  background: #9ca3af;
  cursor: not-allowed;
}

.drop-zone {
  margin-top: 16px;
  padding: 12px;
  border: 2px dashed #d1d5db;
  border-radius: 8px;
  text-align: center;
  color: #6b7280;
  font-size: 0.9rem;
}

@media (prefers-color-scheme: dark) {
  .todo-form {
    background: #1f2937;
  }
  
  .todo-form h2 {
    color: #f9fafb;
  }
  
  .form-group label {
    color: #d1d5db;
  }
  
  .form-input,
  .form-textarea {
    background: #374151;
    border-color: #4b5563;
    color: #f9fafb;
  }
  
  .form-input:focus,
  .form-textarea:focus {
    border-color: #6366f1;
  }
  
  .drop-zone {
    border-color: #4b5563;
    color: #9ca3af;
  }
}
</style>