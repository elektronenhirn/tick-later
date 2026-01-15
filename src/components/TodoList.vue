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

// Time-based categorization for pending todos
const categorizedPendingTodos = computed(() => {
  const now = new Date();
  const in2Hours = new Date(now.getTime() + 2 * 60 * 60 * 1000);
  const in4Hours = new Date(now.getTime() + 4 * 60 * 60 * 1000);
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  tomorrow.setHours(0, 0, 0, 0);
  
  // Calculate next Monday at 7am (same logic as getTimestampForSection)
  const nextWeek = new Date(now);
  const daysUntilMonday = (8 - nextWeek.getDay()) % 7;
  if (daysUntilMonday === 0) {
    // If today is Monday, next Monday is 7 days away
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

function getTimestampForSection(section: string): string {
  const now = new Date();
  
  switch (section) {
    case 'overdue':
      // 1 hour ago to make it overdue
      const oneHourAgo = new Date(now.getTime() - 1 * 60 * 60 * 1000);
      return oneHourAgo.toISOString();
    
    case 'next2Hours':
      // In 1 hour
      const in1Hour = new Date(now.getTime() + 1 * 60 * 60 * 1000);
      return in1Hour.toISOString();
    
    case 'next2To4Hours':
      // In 3 hours
      const in3Hours = new Date(now.getTime() + 3 * 60 * 60 * 1000);
      return in3Hours.toISOString();
    
    case 'tomorrow':
      // Tomorrow at 7am
      const tomorrow = new Date(now);
      tomorrow.setDate(tomorrow.getDate() + 1);
      tomorrow.setHours(7, 0, 0, 0);
      return tomorrow.toISOString();
    
    case 'nextWeek':
      // Monday next week at 7am
      const nextMonday = new Date(now);
      const daysUntilMonday = (8 - nextMonday.getDay()) % 7;
      if (daysUntilMonday === 0) {
        // If today is Monday, next Monday is 7 days away
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
    // This is an existing todo being moved
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
    // This is external text being dropped to create a new todo
    const revisitAt = getTimestampForSection(section);
    emit("addTodo", {
      title: text.trim(),
      revisitAt
    });
  }
  
  // Remove drag-over styling
  const target = event.currentTarget as HTMLElement;
  target.classList.remove('drag-over');
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  
  // Add visual feedback
  const target = event.currentTarget as HTMLElement;
  target.classList.add('drag-over');
}

function handleDragLeave(event: DragEvent) {
  // Remove visual feedback when dragging leaves
  const target = event.currentTarget as HTMLElement;
  target.classList.remove('drag-over');
}
</script>

<template>
  <div class="todo-list">
    <div>
      <!-- Overdue Todos -->
      <div class="todo-section">
        <h3 class="section-title overdue">⚠️ Overdue ({{ overdueTodos.length }})</h3>
        <div 
          v-if="overdueTodos.length > 0" 
          class="overdue-todos drop-zone"
          @drop="handleDrop($event, 'overdue')"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
        >
          <TodoItem
            v-for="todo in overdueTodos"
            :key="todo.id"
            :todo="todo"
            @toggle-complete="handleToggleComplete"
            @delete-todo="handleDeleteTodo"
          />
        </div>
        <div 
          v-else 
          class="empty-section drop-zone"
          @drop="handleDrop($event, 'overdue')"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
        >
          <span>No overdue todos</span>
          <small class="drop-hint">Drop text here to create overdue todo or drag existing todo here</small>
        </div>
      </div>
      
      <!-- Pending Todos by Time Windows -->
      <div class="todo-section">
        <h3 class="section-title">Pending ({{ pendingTodos.length }})</h3>
        
        <div class="time-grid">
          <!-- Upper Left: Next 2 Hours -->
          <div 
            class="time-quadrant drop-zone-quadrant"
            @drop="handleDrop($event, 'next2Hours')"
            @dragover="handleDragOver"
            @dragleave="handleDragLeave"
          >
            <h4 class="subsection-title urgent">⚡ Next 2 Hours ({{ categorizedPendingTodos.next2Hours.length }})</h4>
            <div 
              v-if="categorizedPendingTodos.next2Hours.length > 0" 
              class="quadrant-todos"
              @drop="handleDrop($event, 'next2Hours')"
              @dragover="handleDragOver"
              @dragleave="handleDragLeave"
            >
              <TodoItem
                v-for="todo in categorizedPendingTodos.next2Hours"
                :key="todo.id"
                :todo="todo"
                @toggle-complete="handleToggleComplete"
                @delete-todo="handleDeleteTodo"
              />
            </div>
            <div v-else class="empty-quadrant">
              <span>No urgent todos</span>
              <small class="drop-hint">Drop text here to create todo for 1 hour from now</small>
            </div>
          </div>
          
          <!-- Upper Right: 2-4 Hours -->
          <div 
            class="time-quadrant drop-zone-quadrant"
            @drop="handleDrop($event, 'next2To4Hours')"
            @dragover="handleDragOver"
            @dragleave="handleDragLeave"
          >
            <h4 class="subsection-title soon">🕐 2-4 Hours ({{ categorizedPendingTodos.next2To4Hours.length }})</h4>
            <div 
              v-if="categorizedPendingTodos.next2To4Hours.length > 0" 
              class="quadrant-todos"
              @drop="handleDrop($event, 'next2To4Hours')"
              @dragover="handleDragOver"
              @dragleave="handleDragLeave"
            >
              <TodoItem
                v-for="todo in categorizedPendingTodos.next2To4Hours"
                :key="todo.id"
                :todo="todo"
                @toggle-complete="handleToggleComplete"
                @delete-todo="handleDeleteTodo"
              />
            </div>
            <div v-else class="empty-quadrant">
              <span>No todos in 2-4 hours</span>
              <small class="drop-hint">Drop text here to create todo for 3 hours from now</small>
            </div>
          </div>
          
          <!-- Lower Left: Tomorrow or Later -->
          <div 
            class="time-quadrant drop-zone-quadrant"
            @drop="handleDrop($event, 'tomorrow')"
            @dragover="handleDragOver"
            @dragleave="handleDragLeave"
          >
            <h4 class="subsection-title tomorrow">📅 Tomorrow or Later ({{ categorizedPendingTodos.tomorrow.length }})</h4>
            <div 
              v-if="categorizedPendingTodos.tomorrow.length > 0" 
              class="quadrant-todos"
              @drop="handleDrop($event, 'tomorrow')"
              @dragover="handleDragOver"
              @dragleave="handleDragLeave"
            >
              <TodoItem
                v-for="todo in categorizedPendingTodos.tomorrow"
                :key="todo.id"
                :todo="todo"
                @toggle-complete="handleToggleComplete"
                @delete-todo="handleDeleteTodo"
              />
            </div>
            <div v-else class="empty-quadrant">
              <span>No todos for tomorrow</span>
              <small class="drop-hint">Drop text here to create todo for tomorrow 7AM</small>
            </div>
          </div>
          
          <!-- Lower Right: Next Week or Later -->
          <div 
            class="time-quadrant drop-zone-quadrant"
            @drop="handleDrop($event, 'nextWeek')"
            @dragover="handleDragOver"
            @dragleave="handleDragLeave"
          >
            <h4 class="subsection-title future">📆 Next Week or Later ({{ categorizedPendingTodos.nextWeek.length }})</h4>
            <div 
              v-if="categorizedPendingTodos.nextWeek.length > 0" 
              class="quadrant-todos"
              @drop="handleDrop($event, 'nextWeek')"
              @dragover="handleDragOver"
              @dragleave="handleDragLeave"
            >
              <TodoItem
                v-for="todo in categorizedPendingTodos.nextWeek"
                :key="todo.id"
                :todo="todo"
                @toggle-complete="handleToggleComplete"
                @delete-todo="handleDeleteTodo"
              />
            </div>
            <div v-else class="empty-quadrant">
              <span>No future todos</span>
              <small class="drop-hint">Drop text here to create todo for Monday 7AM</small>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Completed Todos -->
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
  margin: 0 0 20px 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: #374151;
  border-bottom: 2px solid #e5e7eb;
  padding-bottom: 8px;
}

.section-title.overdue {
  color: #dc2626;
  border-bottom-color: #dc2626;
}

.overdue-todos {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.empty-section {
  text-align: center;
  padding: 20px;
  color: #6b7280;
  font-style: italic;
  background: #f9fafb;
  border-radius: 8px;
  border: 2px dashed #e5e7eb;
}

.time-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  margin-bottom: 24px;
}

.time-quadrant {
  background: #f9fafb;
  border-radius: 12px;
  padding: 20px;
  border: 2px solid #e5e7eb;
  min-height: 300px;
  display: flex;
  flex-direction: column;
}

.subsection-title {
  margin: 0 0 16px 0;
  font-size: 1rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 8px;
}

.subsection-title.urgent {
  color: #dc2626;
  border-bottom: 2px solid #dc2626;
}

.subsection-title.soon {
  color: #ea580c;
  border-bottom: 2px solid #ea580c;
}

.subsection-title.tomorrow {
  color: #2563eb;
  border-bottom: 2px solid #2563eb;
}

.subsection-title.future {
  color: #059669;
  border-bottom: 2px solid #059669;
}

.quadrant-todos {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex-grow: 1;
  min-height: 60px;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.quadrant-todos.drag-over {
  background: #f3f4f6;
  border: 2px dashed #4f46e5;
}

.empty-quadrant {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex-grow: 1;
  color: #9ca3af;
  font-style: italic;
  text-align: center;
  padding: 20px;
  gap: 8px;
}

.drop-hint {
  font-size: 0.75rem;
  color: #6b7280;
  margin-top: 4px;
}

.drop-zone,
.drop-zone-quadrant {
  transition: all 0.2s ease;
  cursor: pointer;
}

.drop-zone.drag-over,
.drop-zone-quadrant.drag-over {
  background: #f3f4f6;
  border-color: #4f46e5;
  border-style: dashed;
  transform: scale(1.02);
  box-shadow: 0 4px 12px rgba(79, 70, 229, 0.15);
}

.drop-zone.drag-over .empty-section,
.drop-zone-quadrant.drag-over .empty-quadrant {
  color: #4f46e5;
}

.drop-zone.drag-over .drop-hint,
.drop-zone-quadrant.drag-over .drop-hint {
  color: #4f46e5;
  font-weight: 500;
}

.todos-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

@media (max-width: 1024px) {
  .time-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }
  
  .time-quadrant {
    min-height: 200px;
    padding: 16px;
  }
}

@media (max-width: 768px) {
  .todos-grid {
    grid-template-columns: 1fr;
  }
  
  .time-quadrant {
    min-height: 150px;
    padding: 12px;
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
  
  .section-title.overdue {
    color: #fca5a5;
    border-bottom-color: #dc2626;
  }
  
  .empty-section {
    color: #9ca3af;
    background: #1f2937;
    border-color: #4b5563;
  }
  
  .time-quadrant {
    background: #1f2937;
    border-color: #4b5563;
  }
  
  .subsection-title.urgent {
    color: #fca5a5;
    border-bottom-color: #dc2626;
  }
  
  .subsection-title.soon {
    color: #fed7aa;
    border-bottom-color: #ea580c;
  }
  
  .subsection-title.tomorrow {
    color: #93c5fd;
    border-bottom-color: #2563eb;
  }
  
  .subsection-title.future {
    color: #6ee7b7;
    border-bottom-color: #059669;
  }
  
  .empty-quadrant {
    color: #6b7280;
  }
  
  .drop-hint {
    color: #9ca3af;
  }
  
  .drop-zone.drag-over,
  .drop-zone-quadrant.drag-over {
    background: #374151;
    border-color: #6366f1;
  }
  
  .drop-zone.drag-over .empty-section,
  .drop-zone-quadrant.drag-over .empty-quadrant {
    color: #6366f1;
  }
  
  .drop-zone.drag-over .drop-hint,
  .drop-zone-quadrant.drag-over .drop-hint {
    color: #6366f1;
  }
  
  .quadrant-todos.drag-over {
    background: #374151;
    border-color: #6366f1;
  }
}
</style>