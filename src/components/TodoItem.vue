<script setup lang="ts">
import { computed } from "vue";
import type { Todo } from "../types/todo";
import LinkifiedText from "./LinkifiedText.vue";

const props = defineProps<{
  todo: Todo;
}>();

const emit = defineEmits<{
  toggleComplete: [todoId: string];
  deleteTodo: [todoId: string];
  editTodo: [todo: Todo];
}>();

const revisitDate = computed(() => {
  const date = new Date(props.todo.revisit_at);
  return {
    time: date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' }),
    date: date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' }),
    full: date.toLocaleString()
  };
});

const isOverdue = computed(() => {
  return new Date(props.todo.revisit_at) < new Date() && !props.todo.completed;
});

const createdDate = computed(() => {
  return new Date(props.todo.created_at).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  });
});

function handleToggleComplete() {
  emit("toggleComplete", props.todo.id);
}

function handleDelete() {
  if (confirm("Are you sure you want to delete this entry?")) {
    emit("deleteTodo", props.todo.id);
  }
}

function handleEdit() {
  emit("editTodo", props.todo);
}

function handleDragStart(event: DragEvent) {
  if (event.dataTransfer) {
    event.dataTransfer.setData("text/plain", props.todo.title);
    event.dataTransfer.setData("application/json", JSON.stringify(props.todo));
    event.dataTransfer.effectAllowed = "move";
  }
}
</script>

<template>
  <article
    class="todo-item"
    :class="{
      'todo-item--completed': todo.completed,
      'todo-item--overdue': isOverdue
    }"
    draggable="true"
    @dragstart="handleDragStart"
    @dblclick="handleEdit"
  >
    <div class="item-body">
      <header class="item-header">
        <button
          class="checkbox"
          :class="{ 'checkbox--checked': todo.completed }"
          @click="handleToggleComplete"
          :title="todo.completed ? 'Mark as pending' : 'Mark as complete'"
        >
          <svg v-if="todo.completed" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
        </button>

        <h3 class="item-title">
          <LinkifiedText :text="todo.title" />
        </h3>

        <div class="item-actions">
          <button class="action-btn action-btn--edit" @click="handleEdit" title="Edit entry">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button class="action-btn action-btn--delete" @click="handleDelete" title="Delete entry">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
          <div class="drag-grip" title="Drag to reschedule">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      </header>

      <div class="item-content" v-if="todo.description">
        <p class="item-description">
          <LinkifiedText :text="todo.description" />
        </p>
      </div>

      <footer class="item-footer">
        <time class="schedule-time" :datetime="todo.revisit_at" :title="revisitDate.full">
          DUE {{ revisitDate.date }} at {{ revisitDate.time }}
        </time>
        <time class="created-date">ADDED {{ createdDate }}</time>
      </footer>
    </div>
  </article>
</template>

<style scoped>
.todo-item {
  display: flex;
  width: 100%;
  box-sizing: border-box;
  background: var(--paper);
  border: 1px solid var(--rule-line);
  border-radius: 8px;
  overflow: hidden;
  position: relative;
  cursor: grab;
  transition: all 0.2s ease;
}

.todo-item:hover {
  border-color: var(--ink-light);
  box-shadow: 2px 2px 0 var(--ink-shadow);
  transform: translate(-1px, -1px);
}

.todo-item:active {
  cursor: grabbing;
  transform: translate(0, 0);
  box-shadow: 1px 1px 0 var(--ink-shadow);
}

.item-body {
  flex: 1;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.item-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.checkbox {
  width: 22px;
  height: 22px;
  flex-shrink: 0;
  border: 2px solid var(--ink-light);
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  color: var(--paper);
  padding: 0;
}

.checkbox:hover {
  border-color: var(--ink);
}

.checkbox--checked {
  background: var(--success);
  border-color: var(--success);
}

.item-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.action-btn {
  width: 28px;
  height: 28px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-light);
  padding: 0;
  transition: all 0.15s ease;
  opacity: 0;
}

.todo-item:hover .action-btn {
  opacity: 1;
}

.action-btn:hover {
  color: var(--ink);
  border-color: var(--ink);
}

.action-btn--edit:hover {
  color: var(--accent);
  border-color: var(--accent);
}

.action-btn--delete:hover {
  color: var(--error);
  border-color: var(--error);
}

.drag-grip {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 6px 4px;
  cursor: grab;
  opacity: 0.4;
  transition: opacity 0.15s;
}

.todo-item:hover .drag-grip {
  opacity: 0.8;
}

.drag-grip span {
  width: 12px;
  height: 2px;
  background: var(--ink-light);
  border-radius: 1px;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-title {
  flex: 1;
  min-width: 0;
  font-family: var(--font-body);
  font-size: 1rem;
  font-weight: 600;
  color: var(--ink);
  margin: 0;
  line-height: 1.4;
  word-wrap: break-word;
}

.todo-item--completed .item-title {
  text-decoration: line-through;
  color: var(--ink-light);
}

.item-description {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--ink-light);
  margin: 0;
  line-height: 1.5;
  word-wrap: break-word;
}

.todo-item--completed .item-description {
  opacity: 0.6;
}

.item-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding-top: 8px;
  border-top: 1px dashed var(--rule-line);
}

.schedule-time {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--ink);
}

.created-date {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--ink-light);
}

/* Completed state */
.todo-item--completed {
  opacity: 0.7;
}

.todo-item--completed .item-footer {
  border-top-color: transparent;
}


/* Dark mode adjustments are handled by CSS variables in App.vue */
</style>
