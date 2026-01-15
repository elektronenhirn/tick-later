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
  addTodo: [todoData: { title: string; description?: string; revisitAt: string }];
  updateTodo: [todoData: { id: string; revisitAt: string }];
  editTodo: [todo: Todo];
}>();

const sortedTodos = computed(() => {
  return [...props.todos].sort((a, b) => {
    if (a.completed !== b.completed) {
      return a.completed ? 1 : -1;
    }
    return new Date(a.revisit_at).getTime() - new Date(b.revisit_at).getTime();
  });
});

const now = new Date();
const overdueTodos = computed(() =>
  sortedTodos.value.filter(t => !t.completed && new Date(t.revisit_at) < now)
);
const pendingTodos = computed(() =>
  sortedTodos.value.filter(t => !t.completed && new Date(t.revisit_at) >= now)
);
const completedTodos = computed(() => sortedTodos.value.filter(t => t.completed));

const categorizedPendingTodos = computed(() => {
  const now = new Date();
  const in2Hours = new Date(now.getTime() + 2 * 60 * 60 * 1000);
  const in4Hours = new Date(now.getTime() + 4 * 60 * 60 * 1000);
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  tomorrow.setHours(0, 0, 0, 0);

  const nextWeek = new Date(now);
  const daysUntilMonday = (8 - nextWeek.getDay()) % 7;
  if (daysUntilMonday === 0) {
    nextWeek.setDate(nextWeek.getDate() + 7);
  } else {
    nextWeek.setDate(nextWeek.getDate() + daysUntilMonday);
  }
  nextWeek.setHours(7, 0, 0, 0);

  const categories = {
    next2Hours: [] as Todo[],
    next2To4Hours: [] as Todo[],
    tomorrow: [] as Todo[],
    nextWeek: [] as Todo[]
  };

  pendingTodos.value.forEach(todo => {
    const revisitDate = new Date(todo.revisit_at);

    if (revisitDate <= in2Hours) {
      categories.next2Hours.push(todo);
    } else if (revisitDate <= in4Hours) {
      categories.next2To4Hours.push(todo);
    } else if (revisitDate < nextWeek) {
      categories.tomorrow.push(todo);
    } else {
      categories.nextWeek.push(todo);
    }
  });

  return categories;
});

function handleToggleComplete(todoId: string) {
  emit("toggleComplete", todoId);
}

function handleDeleteTodo(todoId: string) {
  emit("deleteTodo", todoId);
}

function handleEditTodo(todo: Todo) {
  emit("editTodo", todo);
}

function getTimestampForSection(section: string): string {
  const now = new Date();

  switch (section) {
    case 'overdue':
      const oneHourAgo = new Date(now.getTime() - 1 * 60 * 60 * 1000);
      return oneHourAgo.toISOString();

    case 'next2Hours':
      const in1Hour = new Date(now.getTime() + 1 * 60 * 60 * 1000);
      return in1Hour.toISOString();

    case 'next2To4Hours':
      const in3Hours = new Date(now.getTime() + 3 * 60 * 60 * 1000);
      return in3Hours.toISOString();

    case 'tomorrow':
      const tomorrow = new Date(now);
      tomorrow.setDate(tomorrow.getDate() + 1);
      tomorrow.setHours(7, 0, 0, 0);
      return tomorrow.toISOString();

    case 'nextWeek':
      const nextMonday = new Date(now);
      const daysUntilMonday = (8 - nextMonday.getDay()) % 7;
      if (daysUntilMonday === 0) {
        nextMonday.setDate(nextMonday.getDate() + 7);
      } else {
        nextMonday.setDate(nextMonday.getDate() + daysUntilMonday);
      }
      nextMonday.setHours(7, 0, 0, 0);
      return nextMonday.toISOString();

    default:
      return now.toISOString();
  }
}

function handleDrop(event: DragEvent, section: string) {
  event.preventDefault();

  const todoJson = event.dataTransfer?.getData("application/json");
  const text = event.dataTransfer?.getData("text/plain");

  if (todoJson) {
    try {
      const todo = JSON.parse(todoJson);
      const newRevisitAt = getTimestampForSection(section);
      emit("updateTodo", {
        id: todo.id,
        revisitAt: newRevisitAt
      });
    } catch (e) {
      console.error("Failed to parse todo JSON:", e);
    }
  } else if (text && text.trim()) {
    const revisitAt = getTimestampForSection(section);
    emit("addTodo", {
      title: text.trim(),
      revisitAt
    });
  }

  const target = event.currentTarget as HTMLElement;
  target.classList.remove('drag-over');
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  const target = event.currentTarget as HTMLElement;
  target.classList.add('drag-over');
}

function handleDragLeave(event: DragEvent) {
  const target = event.currentTarget as HTMLElement;
  target.classList.remove('drag-over');
}
</script>

<template>
  <div class="todo-list">
    <!-- Overdue Section -->
    <section class="section section--overdue" v-if="overdueTodos.length > 0">
      <header class="section-header">
        <div class="section-badge section-badge--overdue">OVERDUE</div>
        <h2 class="section-title">Requires Attention</h2>
        <span class="section-count">{{ overdueTodos.length }} {{ overdueTodos.length === 1 ? 'item' : 'items' }}</span>
      </header>
      <div
        class="section-content drop-zone"
        @drop="handleDrop($event, 'overdue')"
        @dragover="handleDragOver"
        @dragleave="handleDragLeave"
      >
        <TransitionGroup name="list" tag="div" class="items-list">
          <TodoItem
            v-for="todo in overdueTodos"
            :key="todo.id"
            :todo="todo"
            @toggle-complete="handleToggleComplete"
            @delete-todo="handleDeleteTodo"
            @edit-todo="handleEditTodo"
          />
        </TransitionGroup>
      </div>
    </section>

    <!-- Main Time Grid -->
    <section class="section">
      <header class="section-header">
        <h2 class="section-title">Scheduled</h2>
        <span class="section-count">{{ pendingTodos.length }} {{ pendingTodos.length === 1 ? 'item' : 'items' }} pending</span>
      </header>

      <div class="time-grid">
        <!-- Urgent: Next 2 Hours -->
        <div
          class="time-cell time-cell--urgent drop-zone"
          @drop="handleDrop($event, 'next2Hours')"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
        >
          <header class="cell-header">
            <span class="cell-label">Next 2 Hours</span>
            <span class="cell-count" v-if="categorizedPendingTodos.next2Hours.length">{{ categorizedPendingTodos.next2Hours.length }}</span>
          </header>
          <div class="cell-content">
            <TransitionGroup name="list" tag="div" class="items-column" v-if="categorizedPendingTodos.next2Hours.length">
              <TodoItem
                v-for="todo in categorizedPendingTodos.next2Hours"
                :key="todo.id"
                :todo="todo"
                @toggle-complete="handleToggleComplete"
                @delete-todo="handleDeleteTodo"
                @edit-todo="handleEditTodo"
              />
            </TransitionGroup>
            <div v-else class="empty-cell">
              <span class="empty-text">Nothing urgent</span>
              <span class="drop-hint">Drop to schedule for 1hr from now</span>
            </div>
          </div>
        </div>

        <!-- Soon: 2-4 Hours -->
        <div
          class="time-cell time-cell--soon drop-zone"
          @drop="handleDrop($event, 'next2To4Hours')"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
        >
          <header class="cell-header">
            <span class="cell-label">2-4 Hours</span>
            <span class="cell-count" v-if="categorizedPendingTodos.next2To4Hours.length">{{ categorizedPendingTodos.next2To4Hours.length }}</span>
          </header>
          <div class="cell-content">
            <TransitionGroup name="list" tag="div" class="items-column" v-if="categorizedPendingTodos.next2To4Hours.length">
              <TodoItem
                v-for="todo in categorizedPendingTodos.next2To4Hours"
                :key="todo.id"
                :todo="todo"
                @toggle-complete="handleToggleComplete"
                @delete-todo="handleDeleteTodo"
                @edit-todo="handleEditTodo"
              />
            </TransitionGroup>
            <div v-else class="empty-cell">
              <span class="empty-text">Clear ahead</span>
              <span class="drop-hint">Drop to schedule for 3hrs from now</span>
            </div>
          </div>
        </div>

        <!-- Tomorrow -->
        <div
          class="time-cell time-cell--tomorrow drop-zone"
          @drop="handleDrop($event, 'tomorrow')"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
        >
          <header class="cell-header">
            <span class="cell-label">Tomorrow & Later</span>
            <span class="cell-count" v-if="categorizedPendingTodos.tomorrow.length">{{ categorizedPendingTodos.tomorrow.length }}</span>
          </header>
          <div class="cell-content">
            <TransitionGroup name="list" tag="div" class="items-column" v-if="categorizedPendingTodos.tomorrow.length">
              <TodoItem
                v-for="todo in categorizedPendingTodos.tomorrow"
                :key="todo.id"
                :todo="todo"
                @toggle-complete="handleToggleComplete"
                @delete-todo="handleDeleteTodo"
                @edit-todo="handleEditTodo"
              />
            </TransitionGroup>
            <div v-else class="empty-cell">
              <span class="empty-text">Tomorrow is free</span>
              <span class="drop-hint">Drop to schedule for tomorrow 7AM</span>
            </div>
          </div>
        </div>

        <!-- Next Week -->
        <div
          class="time-cell time-cell--future drop-zone"
          @drop="handleDrop($event, 'nextWeek')"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
        >
          <header class="cell-header">
            <span class="cell-label">Next Week +</span>
            <span class="cell-count" v-if="categorizedPendingTodos.nextWeek.length">{{ categorizedPendingTodos.nextWeek.length }}</span>
          </header>
          <div class="cell-content">
            <TransitionGroup name="list" tag="div" class="items-column" v-if="categorizedPendingTodos.nextWeek.length">
              <TodoItem
                v-for="todo in categorizedPendingTodos.nextWeek"
                :key="todo.id"
                :todo="todo"
                @toggle-complete="handleToggleComplete"
                @delete-todo="handleDeleteTodo"
                @edit-todo="handleEditTodo"
              />
            </TransitionGroup>
            <div v-else class="empty-cell">
              <span class="empty-text">Week ahead is open</span>
              <span class="drop-hint">Drop to schedule for Monday 7AM</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Completed Section -->
    <section class="section section--completed" v-if="completedTodos.length > 0">
      <header class="section-header">
        <div class="section-badge section-badge--completed">DONE</div>
        <h2 class="section-title">Completed</h2>
        <span class="section-count">{{ completedTodos.length }} {{ completedTodos.length === 1 ? 'item' : 'items' }}</span>
      </header>
      <div class="section-content">
        <TransitionGroup name="list" tag="div" class="items-grid">
          <TodoItem
            v-for="todo in completedTodos"
            :key="todo.id"
            :todo="todo"
            @toggle-complete="handleToggleComplete"
            @delete-todo="handleDeleteTodo"
            @edit-todo="handleEditTodo"
          />
        </TransitionGroup>
      </div>
    </section>
  </div>
</template>

<style scoped>
.todo-list {
  display: flex;
  flex-direction: column;
  gap: 48px;
}

/* Section Styles */
.section {
  position: relative;
}

.section-header {
  display: flex;
  align-items: baseline;
  gap: 16px;
  margin-bottom: 24px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--rule-line);
}

.section-badge {
  font-family: var(--font-mono);
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  padding: 5px 10px;
  border: 2px solid currentColor;
  transform: rotate(-1deg);
}

.section-badge--overdue {
  color: var(--urgent);
}

.section-badge--completed {
  color: var(--success);
}

.section-title {
  font-family: var(--font-display);
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: -0.02em;
  margin: 0;
}

.section-count {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--ink-light);
  margin-left: auto;
}

.section-content {
  min-height: 80px;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}

.items-list > * {
  width: 100%;
}

.items-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

/* Overdue styling */
.section--overdue .section-header {
  border-bottom-color: var(--urgent);
}

.section--overdue .section-title {
  color: var(--urgent);
}

/* Completed styling */
.section--completed {
  opacity: 0.8;
}

.section--completed .section-header {
  border-bottom-color: var(--success);
}

/* Time Grid */
.time-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
}

.time-cell {
  background: var(--paper-alt);
  border: 2px solid var(--rule-line);
  padding: 20px;
  min-height: 280px;
  min-width: 0; /* Prevents content from stretching the grid cell */
  display: flex;
  flex-direction: column;
  transition: all 0.2s ease;
  position: relative;
}

.time-cell::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
}

.time-cell--urgent::before {
  background: var(--urgent);
}

.time-cell--soon::before {
  background: var(--soon);
}

.time-cell--tomorrow::before {
  background: var(--tomorrow);
}

.time-cell--future::before {
  background: var(--future);
}

.cell-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px dashed var(--rule-line);
}

.cell-label {
  font-family: var(--font-display);
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: -0.01em;
}

.time-cell--urgent .cell-label {
  color: var(--urgent);
}

.time-cell--soon .cell-label {
  color: var(--soon);
}

.time-cell--tomorrow .cell-label {
  color: var(--tomorrow);
}

.time-cell--future .cell-label {
  color: var(--future);
}

.cell-count {
  font-family: var(--font-mono);
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--ink-light);
  background: var(--paper);
  padding: 2px 8px;
  border: 1px solid var(--rule-line);
}

.cell-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.items-column {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}

.items-column > * {
  width: 100%;
}

.empty-cell {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 8px;
  padding: 24px;
}

.empty-text {
  font-family: var(--font-body);
  font-size: 1rem;
  font-style: italic;
  color: var(--ink-light);
}

.drop-hint {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  color: var(--ink-light);
  opacity: 0.7;
  letter-spacing: 0.02em;
}

/* Drop Zone Interactions */
.drop-zone {
  transition: all 0.2s ease;
}

.drop-zone.drag-over {
  border-style: dashed;
  border-color: var(--accent);
  background: var(--paper);
  transform: scale(1.01);
  box-shadow: 0 4px 20px var(--ink-shadow);
}

.drop-zone.drag-over .empty-text,
.drop-zone.drag-over .drop-hint {
  color: var(--accent);
}

/* List Transitions */
.list-enter-active {
  transition: all 0.3s ease;
}

.list-leave-active {
  transition: all 0.2s ease;
  position: absolute;
}

.list-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.list-move {
  transition: transform 0.3s ease;
}

/* Responsive */
@media (max-width: 1024px) {
  .time-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .time-cell {
    min-height: 200px;
    padding: 16px;
  }
}

@media (max-width: 768px) {
  .section-header {
    flex-wrap: wrap;
    gap: 8px 16px;
  }

  .section-count {
    width: 100%;
    margin-left: 0;
    order: 3;
  }

  .items-grid {
    grid-template-columns: 1fr;
  }

  .time-cell {
    min-height: 160px;
  }
}
</style>
