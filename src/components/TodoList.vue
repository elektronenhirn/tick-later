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

// Time-based categorization for pending todos
const categorizedPendingTodos = computed(() => {
  const now = new Date();
  const in2Hours = new Date(now.getTime() + 2 * 60 * 60 * 1000);
  const in4Hours = new Date(now.getTime() + 4 * 60 * 60 * 1000);
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  tomorrow.setHours(0, 0, 0, 0);
  const nextWeek = new Date(now);
  nextWeek.setDate(nextWeek.getDate() + 7);

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
</script>

<template>
  <div class="todo-list">
    <div v-if="todos.length === 0" class="empty-state">
      <h3>No todos yet</h3>
      <p>Add your first todo above to get started!</p>
    </div>
    
    <div v-else>
      <!-- Pending Todos by Time Windows -->
      <div v-if="pendingTodos.length > 0" class="todo-section">
        <h3 class="section-title">Pending ({{ pendingTodos.length }})</h3>
        
        <!-- Next 2 Hours -->
        <div v-if="categorizedPendingTodos.next2Hours.length > 0" class="time-subsection">
          <h4 class="subsection-title urgent">⚡ Next 2 Hours ({{ categorizedPendingTodos.next2Hours.length }})</h4>
          <div class="todos-grid">
            <TodoItem
              v-for="todo in categorizedPendingTodos.next2Hours"
              :key="todo.id"
              :todo="todo"
              @toggle-complete="handleToggleComplete"
              @delete-todo="handleDeleteTodo"
            />
          </div>
        </div>
        
        <!-- 2-4 Hours -->
        <div v-if="categorizedPendingTodos.next2To4Hours.length > 0" class="time-subsection">
          <h4 class="subsection-title soon">🕐 2-4 Hours ({{ categorizedPendingTodos.next2To4Hours.length }})</h4>
          <div class="todos-grid">
            <TodoItem
              v-for="todo in categorizedPendingTodos.next2To4Hours"
              :key="todo.id"
              :todo="todo"
              @toggle-complete="handleToggleComplete"
              @delete-todo="handleDeleteTodo"
            />
          </div>
        </div>
        
        <!-- Tomorrow or Later (but within week) -->
        <div v-if="categorizedPendingTodos.tomorrow.length > 0" class="time-subsection">
          <h4 class="subsection-title tomorrow">📅 Tomorrow or Later ({{ categorizedPendingTodos.tomorrow.length }})</h4>
          <div class="todos-grid">
            <TodoItem
              v-for="todo in categorizedPendingTodos.tomorrow"
              :key="todo.id"
              :todo="todo"
              @toggle-complete="handleToggleComplete"
              @delete-todo="handleDeleteTodo"
            />
          </div>
        </div>
        
        <!-- Next Week or Later -->
        <div v-if="categorizedPendingTodos.nextWeek.length > 0" class="time-subsection">
          <h4 class="subsection-title future">📆 Next Week or Later ({{ categorizedPendingTodos.nextWeek.length }})</h4>
          <div class="todos-grid">
            <TodoItem
              v-for="todo in categorizedPendingTodos.nextWeek"
              :key="todo.id"
              :todo="todo"
              @toggle-complete="handleToggleComplete"
              @delete-todo="handleDeleteTodo"
            />
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

.time-subsection {
  margin-bottom: 24px;
  padding-left: 16px;
  border-left: 3px solid #e5e7eb;
}

.subsection-title {
  margin: 0 0 12px 0;
  font-size: 1rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 8px;
}

.subsection-title.urgent {
  color: #dc2626;
  border-left: 3px solid #dc2626;
  padding-left: 8px;
}

.subsection-title.soon {
  color: #ea580c;
  border-left: 3px solid #ea580c;
  padding-left: 8px;
}

.subsection-title.tomorrow {
  color: #2563eb;
  border-left: 3px solid #2563eb;
  padding-left: 8px;
}

.subsection-title.future {
  color: #059669;
  border-left: 3px solid #059669;
  padding-left: 8px;
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
  
  .time-subsection {
    padding-left: 12px;
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
  
  .time-subsection {
    border-left-color: #4b5563;
  }
  
  .subsection-title.urgent {
    color: #fca5a5;
    border-left-color: #dc2626;
  }
  
  .subsection-title.soon {
    color: #fed7aa;
    border-left-color: #ea580c;
  }
  
  .subsection-title.tomorrow {
    color: #93c5fd;
    border-left-color: #2563eb;
  }
  
  .subsection-title.future {
    color: #6ee7b7;
    border-left-color: #059669;
  }
}
</style>