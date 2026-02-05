<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import type { Todo } from "../types/todo";
import LinkifiedText from "./LinkifiedText.vue";

const props = defineProps<{
  todo: Todo;
  isHighlighted?: boolean;
  isTerminalBusy?: boolean;
}>();

const emit = defineEmits<{
  toggleComplete: [todoId: string];
  deleteTodo: [todoId: string];
  editTodo: [todo: Todo];
  moveTodo: [todoId: string, section: string];
  openTerminal: [todo: Todo];
  switchDesktop: [desktop: number];
  launchWorkspace: [todo: Todo];
}>();

// Context menu state
const showContextMenu = ref(false);
const menuPosition = ref({ x: 0, y: 0 });

// Description expansion state
const isDescriptionExpanded = ref(false);

function toggleDescriptionExpanded() {
  isDescriptionExpanded.value = !isDescriptionExpanded.value;
}

const menuOptions = [
  { label: "Next 2 Hours", section: "next2Hours" },
  { label: "2-4 Hours", section: "next2To4Hours" },
  { label: "Tomorrow & Later", section: "tomorrow" },
  { label: "Next Week +", section: "nextWeek" },
];

function handleContextMenu(event: MouseEvent) {
  event.preventDefault();
  menuPosition.value = { x: event.clientX, y: event.clientY };
  showContextMenu.value = true;
}

function handleMoveToSection(section: string) {
  emit("moveTodo", props.todo.id, section);
  showContextMenu.value = false;
}

// Close menu when clicking outside
function handleClickOutside() {
  if (showContextMenu.value) {
    showContextMenu.value = false;
  }
}

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
});

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

// Premium colored pencil palette - inspired by artist-grade pencil sets
// 64 colors organized by hue with rich, sophisticated tones
const colorPalette = [
  // Scarlets & Crimsons
  '#c23b22', '#a63d40', '#8b2942', '#6b1d2b',
  // Rose & Blush
  '#c9556d', '#b5485d', '#9e3a52', '#8a2846',
  // Magenta & Berry
  '#a64d79', '#8e3f6b', '#763458', '#5e2a48',
  // Violet & Plum
  '#7c5295', '#6b4483', '#583874', '#472d62',
  // Royal & Iris
  '#5c5da8', '#4e509a', '#424589', '#363a78',
  // Ultramarine & Cobalt
  '#3d5a99', '#345089', '#2c4679', '#243b68',
  // Cerulean & Azure
  '#4a7fb5', '#3d71a5', '#326395', '#285685',
  // Teal & Ocean
  '#3d8b8b', '#347b7b', '#2b6b6b', '#225b5b',
  // Viridian & Emerald
  '#3d8b6b', '#347b5d', '#2b6b50', '#225b43',
  // Forest & Hunter
  '#4a7a4a', '#3d6b3d', '#325c32', '#284d28',
  // Olive & Moss
  '#6b7a3d', '#5d6b34', '#505c2b', '#434d22',
  // Ochre & Sienna
  '#b5854a', '#a57540', '#956536', '#85552d',
  // Amber & Bronze
  '#c98b3d', '#b97d34', '#a96f2b', '#996122',
  // Tangerine & Rust
  '#c97a4a', '#b96a3d', '#a95a32', '#994a28',
  // Coral & Terracotta
  '#c96a5a', '#b95a4d', '#a94a40', '#993a34',
  // Warm Grays & Sepia
  '#7a6b5d', '#6b5d50', '#5c5043', '#4d4336',
];

const hatchColor = computed(() => {
  // Use custom color if set
  if (props.todo.color) {
    return props.todo.color;
  }

  // Otherwise, generate from ID hash
  let hash = 0;
  for (let i = 0; i < props.todo.id.length; i++) {
    const char = props.todo.id.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }

  return colorPalette[Math.abs(hash) % colorPalette.length];
});

// Unique hatch angle per todo (0-360 degrees)
const hatchAngle = computed(() => {
  let hash = 0;
  for (let i = 0; i < props.todo.id.length; i++) {
    const char = props.todo.id.charCodeAt(i);
    hash = ((hash << 7) - hash) + char;
    hash = hash & hash;
  }
  return Math.abs(hash) % 360;
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

function handleOpenTerminal() {
  emit("openTerminal", props.todo);
}

function handleSwitchDesktop() {
  if (props.todo.virtual_desktop !== undefined) {
    emit("switchDesktop", props.todo.virtual_desktop);
  }
}

function handleLaunchWorkspace() {
  emit("launchWorkspace", props.todo);
}
</script>

<template>
  <article
    class="todo-item"
    :class="{
      'todo-item--completed': todo.completed,
      'todo-item--overdue': isOverdue,
      'todo-item--highlighted': isHighlighted
    }"
    :data-todo-id="todo.id"
    draggable="true"
    @dragstart="handleDragStart"
    @dblclick="handleEdit"
    @contextmenu="handleContextMenu"
  >
    <!-- Left sidepanel with diagonal hatching -->
    <div class="sidepanel" :style="{ '--hatch-color': hatchColor }">
      <svg class="sidepanel-hatch" viewBox="0 0 28 100" preserveAspectRatio="none">
        <defs>
          <pattern :id="`diagonal-hatch-${todo.id}`" patternUnits="userSpaceOnUse" width="6" height="6" :patternTransform="`rotate(${hatchAngle})`">
            <line x1="0" y1="0" x2="0" y2="6" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
          </pattern>
          <filter :id="`sketchy-sidepanel-${todo.id}`" x="-20%" y="-5%" width="140%" height="110%">
            <feTurbulence type="turbulence" baseFrequency="0.08" numOctaves="2" result="noise" seed="7"/>
            <feDisplacementMap in="SourceGraphic" in2="noise" scale="1.5" xChannelSelector="R" yChannelSelector="G"/>
          </filter>
        </defs>
        <rect x="0" y="0" width="28" height="100" :fill="`url(#diagonal-hatch-${todo.id})`" :filter="`url(#sketchy-sidepanel-${todo.id})`"/>
      </svg>
    </div>

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

        <h3 class="item-title" :style="{ '--title-color': hatchColor }">
          <LinkifiedText :text="todo.title" :virtualDesktop="todo.virtual_desktop" />
          <span v-if="isTerminalBusy" class="terminal-busy-indicator" title="Terminal is executing a command">
            <span class="busy-dot"></span>
          </span>
          <span v-else-if="isTerminalBusy === false" class="terminal-idle-indicator" title="Terminal is idle">
            <span class="idle-dot"></span>
          </span>
          <svg class="title-underline" viewBox="0 0 100 6" preserveAspectRatio="none">
            <filter id="sketch-underline" x="-10%" y="-50%" width="120%" height="200%">
              <feTurbulence type="turbulence" baseFrequency="0.04" numOctaves="2" result="noise" seed="3"/>
              <feDisplacementMap in="SourceGraphic" in2="noise" scale="3" xChannelSelector="R" yChannelSelector="G"/>
            </filter>
            <line x1="0" y1="3" x2="100" y2="3.5" filter="url(#sketch-underline)" />
          </svg>
        </h3>

        <div class="item-actions">
          <button
            v-if="todo.workspace_apps && todo.workspace_apps.length > 0"
            class="action-btn action-btn--workspace"
            @click="handleLaunchWorkspace"
            :title="`Launch workspace (${todo.workspace_apps.length} app${todo.workspace_apps.length > 1 ? 's' : ''})`"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="5 3 19 12 5 21 5 3"/>
            </svg>
          </button>
          <button
            v-if="todo.virtual_desktop !== undefined"
            class="action-btn action-btn--desktop"
            @click="handleSwitchDesktop"
            :title="`Switch to Desktop ${todo.virtual_desktop + 1}`"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
              <line x1="8" y1="21" x2="16" y2="21"/>
              <line x1="12" y1="17" x2="12" y2="21"/>
            </svg>
            <span class="desktop-number">{{ todo.virtual_desktop + 1 }}</span>
          </button>
          <button class="action-btn action-btn--terminal" @click="handleOpenTerminal" title="Open terminal">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="4 17 10 11 4 5"/>
              <line x1="12" y1="19" x2="20" y2="19"/>
            </svg>
          </button>
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

      <div
        class="item-content"
        v-if="todo.description"
        :class="{ 'item-content--collapsed': !isDescriptionExpanded }"
        @click.stop="toggleDescriptionExpanded"
        :title="isDescriptionExpanded ? 'Click to collapse' : 'Click to expand'"
      >
        <p class="item-description">
          <LinkifiedText :text="todo.description" :virtualDesktop="todo.virtual_desktop" />
        </p>
      </div>

      <footer class="item-footer">
        <time class="schedule-time" :datetime="todo.revisit_at" :title="revisitDate.full">
          DUE {{ revisitDate.date }} at {{ revisitDate.time }}
        </time>
        <time class="created-date">ADDED {{ createdDate }}</time>
      </footer>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="showContextMenu"
        class="context-menu"
        :style="{ left: menuPosition.x + 'px', top: menuPosition.y + 'px' }"
        @click.stop
      >
        <div class="context-menu-header">Move to...</div>
        <button
          v-for="option in menuOptions"
          :key="option.section"
          class="context-menu-item"
          @click="handleMoveToSection(option.section)"
        >
          {{ option.label }}
        </button>
      </div>
    </Teleport>
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

/* Left sidepanel with diagonal hatching */
.sidepanel {
  width: 28px;
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
  background: color-mix(in srgb, var(--hatch-color) 8%, transparent);
  border-right: 1px solid color-mix(in srgb, var(--hatch-color) 20%, var(--rule-line));
}

.sidepanel-hatch {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  color: var(--hatch-color);
  opacity: 0.5;
  transition: opacity 0.3s ease;
}

.todo-item:hover .sidepanel-hatch {
  opacity: 0.7;
}

.todo-item--completed .sidepanel {
  opacity: 0.5;
  filter: saturate(0.6);
}

/* Highlighted state (from search) */
.todo-item--highlighted {
  animation: highlight-pulse 2s ease-out;
  box-shadow: 0 0 0 4px var(--accent) !important;
  border-color: var(--accent) !important;
  background: var(--accent-light);
  background: color-mix(in srgb, var(--accent) 10%, var(--paper));
}

@keyframes highlight-pulse {
  0%, 25% {
    box-shadow: 0 0 0 8px var(--accent);
    transform: scale(1.02);
  }
  50% {
    box-shadow: 0 0 0 4px var(--accent);
    transform: scale(1);
  }
  75% {
    box-shadow: 0 0 0 6px var(--accent);
  }
  100% {
    box-shadow: 0 0 0 4px var(--accent);
    transform: scale(1);
  }
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

.action-btn--terminal:hover {
  color: var(--future);
  border-color: var(--future);
}

.action-btn--workspace {
  opacity: 0.8 !important;
}

.action-btn--workspace:hover {
  opacity: 1 !important;
  color: var(--success);
  border-color: var(--success);
}

.action-btn--desktop {
  display: flex;
  align-items: center;
  gap: 4px;
  width: auto;
  padding: 0 8px;
  opacity: 0.7;
}

.action-btn--desktop:hover {
  color: var(--tomorrow);
  border-color: var(--tomorrow);
}

.desktop-number {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  font-weight: 600;
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
  cursor: pointer;
  transition: all 0.15s ease;
}

.item-content:hover {
  background: var(--paper-alt);
  margin: -4px;
  padding: 4px;
  border-radius: 4px;
}

.item-content--collapsed .item-description {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  white-space: pre-wrap;
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
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.terminal-busy-indicator,
.terminal-idle-indicator {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
}

.busy-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--warning, #f59e0b);
  animation: pulse-busy 1s ease-in-out infinite;
}

.idle-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--success, #10b981);
  opacity: 0.7;
}

@keyframes pulse-busy {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(0.8);
  }
}

.title-underline {
  position: absolute;
  bottom: -2px;
  left: 0;
  width: 100%;
  height: 6px;
  overflow: visible;
}

.title-underline line {
  stroke: var(--title-color);
  stroke-width: 2;
  stroke-linecap: round;
  opacity: 0.6;
}

.todo-item--completed .item-title {
  text-decoration: line-through;
  color: var(--ink-light);
}

.todo-item--completed .title-underline {
  opacity: 0.3;
}

.item-description {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--ink-light);
  margin: 0;
  line-height: 1.5;
  word-wrap: break-word;
  white-space: pre-wrap;
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

/* Context Menu - using :global because it's teleported to body */
:global(.context-menu) {
  position: fixed;
  z-index: 1000;
  background: var(--paper);
  border: 2px solid var(--ink);
  box-shadow: 4px 4px 0 var(--ink-shadow);
  min-width: 160px;
  padding: 4px 0;
}

:global(.context-menu-header) {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--ink-light);
  padding: 8px 14px 6px;
  border-bottom: 1px dashed var(--rule-line);
  margin-bottom: 4px;
}

:global(.context-menu-item) {
  display: block;
  width: 100%;
  padding: 10px 14px;
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--ink);
  background: transparent;
  border: none;
  text-align: left;
  cursor: pointer;
  transition: all 0.1s ease;
}

:global(.context-menu-item:hover) {
  background: var(--paper-alt);
  color: var(--accent);
}
</style>
