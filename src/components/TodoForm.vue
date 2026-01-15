<script setup lang="ts">
import { ref, watch, computed } from "vue";
import type { Todo } from "../types/todo";

const props = defineProps<{
  editingTodo?: Todo;
}>();

const emit = defineEmits<{
  addTodo: [todo: { title: string; description?: string; revisitAt: string }];
  updateTodo: [todo: { id: string; title: string; description?: string; revisitAt: string }];
}>();

const title = ref("");
const description = ref("");
const revisitAt = ref("");

const isEditMode = computed(() => !!props.editingTodo);

// Watch for editingTodo changes to populate the form
watch(() => props.editingTodo, (todo) => {
  if (todo) {
    title.value = todo.title;
    description.value = todo.description || "";
    // Format the date for datetime-local input
    const date = new Date(todo.revisit_at);
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    revisitAt.value = `${year}-${month}-${day}T${hours}:${minutes}`;
  } else {
    resetForm();
  }
}, { immediate: true });

function resetForm() {
  title.value = "";
  description.value = "";
  revisitAt.value = "";
}

function handleSubmit() {
  if (!title.value.trim() || !revisitAt.value) return;

  if (isEditMode.value && props.editingTodo) {
    emit("updateTodo", {
      id: props.editingTodo.id,
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      revisitAt: revisitAt.value
    });
  } else {
    emit("addTodo", {
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      revisitAt: revisitAt.value
    });
  }

  resetForm();
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

// Quick schedule presets
function setQuickSchedule(preset: 'in1h' | 'in3h' | 'tomorrow' | 'nextWeek') {
  const now = new Date();

  switch (preset) {
    case 'in1h':
      now.setHours(now.getHours() + 1);
      break;
    case 'in3h':
      now.setHours(now.getHours() + 3);
      break;
    case 'tomorrow':
      now.setDate(now.getDate() + 1);
      now.setHours(9, 0, 0, 0);
      break;
    case 'nextWeek':
      const daysUntilMonday = (8 - now.getDay()) % 7 || 7;
      now.setDate(now.getDate() + daysUntilMonday);
      now.setHours(9, 0, 0, 0);
      break;
  }

  // Format for datetime-local input
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  const hours = String(now.getHours()).padStart(2, '0');
  const minutes = String(now.getMinutes()).padStart(2, '0');

  revisitAt.value = `${year}-${month}-${day}T${hours}:${minutes}`;
}
</script>

<template>
  <div class="todo-form">
    <form @submit.prevent="handleSubmit" class="form">
      <div class="form-field">
        <label for="title" class="field-label">
          <span class="label-text">Title</span>
          <span class="label-required">required</span>
        </label>
        <input
          id="title"
          v-model="title"
          type="text"
          placeholder="What needs to be done?"
          required
          @drop="handleDrop"
          @dragover="handleDragOver"
          class="field-input"
          autocomplete="off"
        />
      </div>

      <div class="form-field">
        <label for="description" class="field-label">
          <span class="label-text">Notes</span>
          <span class="label-optional">optional</span>
        </label>
        <textarea
          id="description"
          v-model="description"
          placeholder="Additional context or details..."
          rows="3"
          class="field-textarea"
        ></textarea>
      </div>

      <div class="form-field">
        <label for="revisit-at" class="field-label">
          <span class="label-text">Schedule</span>
          <span class="label-required">required</span>
        </label>

        <div class="quick-presets">
          <button type="button" class="preset-btn" @click="setQuickSchedule('in1h')">
            In 1 hour
          </button>
          <button type="button" class="preset-btn" @click="setQuickSchedule('in3h')">
            In 3 hours
          </button>
          <button type="button" class="preset-btn" @click="setQuickSchedule('tomorrow')">
            Tomorrow 9AM
          </button>
          <button type="button" class="preset-btn" @click="setQuickSchedule('nextWeek')">
            Next Monday
          </button>
        </div>

        <input
          id="revisit-at"
          v-model="revisitAt"
          type="datetime-local"
          required
          class="field-input field-input--datetime"
        />
      </div>

      <div class="form-actions">
        <button
          type="submit"
          :disabled="!title.trim() || !revisitAt"
          class="submit-btn"
        >
          <span class="btn-text">{{ isEditMode ? 'Save Changes' : 'Add Entry' }}</span>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path v-if="isEditMode" d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
            <polyline v-if="isEditMode" points="17 21 17 13 7 13 7 21"/>
            <polyline v-if="isEditMode" points="7 3 7 8 15 8"/>
            <path v-if="!isEditMode" d="M5 12h14M12 5l7 7-7 7"/>
          </svg>
        </button>
      </div>
    </form>

    <div v-if="!isEditMode" class="drop-zone-hint">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
        <polyline points="7 10 12 15 17 10"/>
        <line x1="12" y1="15" x2="12" y2="3"/>
      </svg>
      <span>Drop text onto title field to import</span>
    </div>
  </div>
</template>

<style scoped>
.todo-form {
  padding: 24px 28px 28px;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-label {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.label-text {
  font-family: var(--font-display);
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: -0.01em;
}

.label-required,
.label-optional {
  font-family: var(--font-mono);
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.label-required {
  color: var(--urgent);
}

.label-optional {
  color: var(--ink-light);
}

.field-input,
.field-textarea {
  font-family: var(--font-body);
  font-size: 1rem;
  color: var(--ink);
  background: var(--paper);
  border: 2px solid var(--rule-line);
  padding: 14px 16px;
  transition: all 0.15s ease;
  width: 100%;
}

.field-input::placeholder,
.field-textarea::placeholder {
  color: var(--ink-light);
  opacity: 0.7;
}

.field-input:focus,
.field-textarea:focus {
  outline: none;
  border-color: var(--ink);
  box-shadow: 3px 3px 0 var(--ink-shadow);
}

.field-textarea {
  resize: vertical;
  min-height: 80px;
  line-height: 1.6;
}

.field-input--datetime {
  font-family: var(--font-mono);
  font-size: 0.95rem;
}

.quick-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 10px;
}

.preset-btn {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--ink-light);
  background: transparent;
  border: 1px solid var(--rule-line);
  padding: 6px 12px;
  cursor: pointer;
  transition: all 0.15s ease;
  letter-spacing: 0.02em;
}

.preset-btn:hover {
  color: var(--ink);
  border-color: var(--ink);
  background: var(--paper-alt);
}

.form-actions {
  padding-top: 8px;
}

.submit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  width: 100%;
  padding: 16px 24px;
  font-family: var(--font-body);
  font-size: 1rem;
  font-weight: 600;
  color: var(--paper);
  background: var(--ink);
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 3px 3px 0 var(--ink-shadow);
}

.submit-btn:hover:not(:disabled) {
  transform: translate(-2px, -2px);
  box-shadow: 5px 5px 0 var(--ink-shadow);
}

.submit-btn:active:not(:disabled) {
  transform: translate(1px, 1px);
  box-shadow: 1px 1px 0 var(--ink-shadow);
}

.submit-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  box-shadow: none;
}

.drop-zone-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 20px;
  padding: 12px;
  border: 1px dashed var(--rule-line);
  color: var(--ink-light);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
}

@media (max-width: 768px) {
  .todo-form {
    padding: 20px 24px 24px;
  }

  .quick-presets {
    gap: 6px;
  }

  .preset-btn {
    font-size: 0.7rem;
    padding: 5px 10px;
  }
}
</style>
