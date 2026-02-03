<script setup lang="ts">
import { ref, watch, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Todo, DesktopInfo, WorkspaceApp } from "../types/todo";

const props = defineProps<{
  editingTodo?: Todo;
}>();

const emit = defineEmits<{
  addTodo: [todo: { title: string; description?: string; revisitAt: string; virtualDesktop?: number; color?: string; workspaceApps?: WorkspaceApp[] }];
  updateTodo: [todo: { id: string; title: string; description?: string; revisitAt: string; virtualDesktop?: number; clearVirtualDesktop?: boolean; color?: string; clearColor?: boolean; workspaceApps?: WorkspaceApp[]; clearWorkspaceApps?: boolean }];
}>();

const title = ref("");
const description = ref("");
const revisitAt = ref("");
const virtualDesktop = ref<number | null>(null);
const desktopInfo = ref<DesktopInfo | null>(null);
const desktopSupported = ref(false);
const selectedColor = ref<string | null>(null);
const showColorPicker = ref(false);
const workspaceApps = ref<WorkspaceApp[]>([]);
const showWorkspaceApps = ref(false);

// Premium colored pencil palette
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

async function loadDesktopInfo() {
  try {
    desktopInfo.value = await invoke<DesktopInfo>("get_virtual_desktop_info");
    desktopSupported.value = true;
  } catch {
    // Virtual desktop not supported on this platform
    desktopSupported.value = false;
  }
}

onMounted(() => {
  loadDesktopInfo();
});

function getDesktopName(index: number): string {
  if (desktopInfo.value?.names[index]) {
    return desktopInfo.value.names[index];
  }
  return `Desktop ${index + 1}`;
}

const isEditMode = computed(() => !!props.editingTodo);

// Watch for editingTodo changes to populate the form
watch(() => props.editingTodo, (todo) => {
  if (todo) {
    title.value = todo.title;
    description.value = todo.description || "";
    virtualDesktop.value = todo.virtual_desktop ?? null;
    selectedColor.value = todo.color ?? null;
    workspaceApps.value = todo.workspace_apps ? [...todo.workspace_apps] : [];
    showWorkspaceApps.value = workspaceApps.value.length > 0;
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
  virtualDesktop.value = null;
  selectedColor.value = null;
  showColorPicker.value = false;
  workspaceApps.value = [];
  showWorkspaceApps.value = false;
}

function handleSubmit() {
  if (!title.value.trim() || !revisitAt.value) return;

  // Filter out empty workspace apps and clean up working_dir
  const validApps = workspaceApps.value
    .filter(app => app.command.trim())
    .map(app => ({
      command: app.command.trim(),
      working_dir: app.working_dir?.trim() || undefined
    }));

  if (isEditMode.value && props.editingTodo) {
    const hadDesktop = props.editingTodo.virtual_desktop !== undefined;
    const hasDesktop = virtualDesktop.value !== null;
    const hadColor = props.editingTodo.color !== undefined;
    const hasColor = selectedColor.value !== null;
    const hadApps = props.editingTodo.workspace_apps && props.editingTodo.workspace_apps.length > 0;
    const hasApps = validApps.length > 0;
    emit("updateTodo", {
      id: props.editingTodo.id,
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      revisitAt: revisitAt.value,
      virtualDesktop: virtualDesktop.value ?? undefined,
      clearVirtualDesktop: hadDesktop && !hasDesktop,
      color: selectedColor.value ?? undefined,
      clearColor: hadColor && !hasColor,
      workspaceApps: hasApps ? validApps : undefined,
      clearWorkspaceApps: hadApps && !hasApps
    });
  } else {
    emit("addTodo", {
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      revisitAt: revisitAt.value,
      virtualDesktop: virtualDesktop.value ?? undefined,
      color: selectedColor.value ?? undefined,
      workspaceApps: validApps.length > 0 ? validApps : undefined
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

// Workspace apps management
function addWorkspaceApp() {
  workspaceApps.value.push({ command: "", working_dir: undefined });
}

function removeWorkspaceApp(index: number) {
  workspaceApps.value.splice(index, 1);
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
        <label class="field-label">
          <span class="label-text">Color</span>
          <span class="label-optional">optional</span>
        </label>
        <div class="color-selector">
          <button
            type="button"
            class="color-preview"
            @click="showColorPicker = !showColorPicker"
          >
            <span
              class="color-swatch"
              :style="selectedColor ? { background: selectedColor } : {}"
              :class="{ 'color-swatch--empty': !selectedColor }"
            >
              <span v-if="!selectedColor" class="swatch-icon">?</span>
            </span>
            <span class="color-label">{{ selectedColor ? 'Custom color' : 'Random (auto)' }}</span>
            <svg class="chevron" :class="{ 'chevron--open': showColorPicker }" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </button>
          <Transition name="picker">
            <div v-if="showColorPicker" class="color-picker">
              <button
                type="button"
                class="color-option color-option--random"
                :class="{ 'color-option--selected': selectedColor === null }"
                @click="selectedColor = null; showColorPicker = false"
                title="Random"
              >
                <span class="random-icon">?</span>
              </button>
              <button
                v-for="color in colorPalette"
                :key="color"
                type="button"
                class="color-option"
                :class="{ 'color-option--selected': selectedColor === color }"
                :style="{ background: color }"
                :title="color"
                @click="selectedColor = color; showColorPicker = false"
              />
            </div>
          </Transition>
        </div>
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

      <div v-if="desktopSupported && desktopInfo" class="form-field">
        <label for="virtual-desktop" class="field-label">
          <span class="label-text">Virtual Desktop</span>
          <span class="label-optional">optional</span>
        </label>

        <div class="desktop-selector">
          <button
            type="button"
            class="desktop-btn"
            :class="{ 'desktop-btn--active': virtualDesktop === null }"
            @click="virtualDesktop = null"
          >
            None
          </button>
          <button
            v-for="i in desktopInfo.total"
            :key="i - 1"
            type="button"
            class="desktop-btn"
            :class="{
              'desktop-btn--active': virtualDesktop === i - 1,
              'desktop-btn--current': desktopInfo.current === i - 1
            }"
            @click="virtualDesktop = i - 1"
          >
            {{ getDesktopName(i - 1) }}
            <span v-if="desktopInfo.current === i - 1" class="current-indicator" title="Current desktop"></span>
          </button>
        </div>
      </div>

      <div class="form-field">
        <label class="field-label">
          <span class="label-text">Workspace Apps</span>
          <span class="label-optional">optional</span>
        </label>

        <button
          type="button"
          class="workspace-toggle"
          @click="showWorkspaceApps = !showWorkspaceApps"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
            <line x1="8" y1="21" x2="16" y2="21"/>
            <line x1="12" y1="17" x2="12" y2="21"/>
          </svg>
          <span>{{ workspaceApps.length > 0 ? `${workspaceApps.length} app(s) configured` : 'Configure workspace apps' }}</span>
          <svg class="chevron" :class="{ 'chevron--open': showWorkspaceApps }" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 9 12 15 18 9"/>
          </svg>
        </button>

        <Transition name="picker">
          <div v-if="showWorkspaceApps" class="workspace-apps-panel">
            <p class="workspace-hint">Apps to launch when starting this workspace. They will open on the virtual desktop if set.</p>

            <div v-for="(app, index) in workspaceApps" :key="index" class="workspace-app-entry">
              <div class="app-fields">
                <input
                  v-model="app.command"
                  type="text"
                  placeholder="Command (e.g., code .)"
                  class="app-input app-input--command"
                />
                <input
                  v-model="app.working_dir"
                  type="text"
                  placeholder="Working directory (optional)"
                  class="app-input app-input--dir"
                />
              </div>
              <button
                type="button"
                class="app-remove-btn"
                @click="removeWorkspaceApp(index)"
                title="Remove app"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6L6 18M6 6l12 12"/>
                </svg>
              </button>
            </div>

            <button
              type="button"
              class="add-app-btn"
              @click="addWorkspaceApp"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              Add Application
            </button>
          </div>
        </Transition>
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

.desktop-selector {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.desktop-btn {
  position: relative;
  font-family: var(--font-mono);
  font-size: 0.8rem;
  color: var(--ink-light);
  background: transparent;
  border: 2px solid var(--rule-line);
  padding: 8px 14px;
  cursor: pointer;
  transition: all 0.15s ease;
  letter-spacing: 0.02em;
}

.desktop-btn:hover {
  color: var(--ink);
  border-color: var(--ink);
  background: var(--paper-alt);
}

.desktop-btn--active {
  color: var(--paper);
  background: var(--ink);
  border-color: var(--ink);
}

.desktop-btn--active:hover {
  color: var(--paper);
  background: var(--ink-light);
}

.desktop-btn--current::after {
  content: '';
  position: absolute;
  bottom: 4px;
  left: 50%;
  transform: translateX(-50%);
  width: 4px;
  height: 4px;
  background: var(--accent);
  border-radius: 50%;
}

.desktop-btn--active.desktop-btn--current::after {
  background: var(--paper);
}

.current-indicator {
  display: none;
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

/* Color Selector */
.color-selector {
  position: relative;
}

.color-preview {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  background: var(--paper);
  border: 2px solid var(--rule-line);
  cursor: pointer;
  transition: all 0.15s ease;
  color: var(--ink);
}

.color-preview:hover {
  border-color: var(--ink);
}

.color-swatch {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 2px solid var(--rule-line);
  display: flex;
  align-items: center;
  justify-content: center;
}

.color-swatch--empty {
  background: var(--paper-alt);
  border-style: dashed;
}

.swatch-icon {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  color: var(--ink-light);
}

.color-label {
  flex: 1;
  font-family: var(--font-body);
  font-size: 0.9rem;
  text-align: left;
}

.chevron {
  transition: transform 0.2s ease;
  color: var(--ink-light);
}

.chevron--open {
  transform: rotate(180deg);
}

.color-picker {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--paper);
  border: 2px solid var(--ink);
  box-shadow: 4px 4px 0 var(--ink-shadow);
  z-index: 100;
  margin-top: 4px;
  padding: 8px;
  display: grid;
  grid-template-columns: repeat(13, 1fr);
  gap: 3px;
}

.color-option {
  width: 100%;
  aspect-ratio: 1;
  border: 1px solid transparent;
  border-radius: 2px;
  cursor: pointer;
  transition: all 0.1s ease;
}

.color-option:hover {
  transform: scale(1.2);
  z-index: 1;
  border-color: var(--ink);
}

.color-option--selected {
  border-color: var(--ink);
  box-shadow: 0 0 0 1px var(--paper), 0 0 0 2px var(--ink);
}

.color-option--random {
  background: var(--paper-alt);
  border: 2px dashed var(--rule-line);
  display: flex;
  align-items: center;
  justify-content: center;
}

.color-option--random:hover {
  border-color: var(--ink);
}

.random-icon {
  font-family: var(--font-mono);
  font-size: 0.9rem;
  color: var(--ink-light);
}

/* Picker Transitions */
.picker-enter-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.picker-leave-active {
  transition: opacity 0.1s ease, transform 0.1s ease;
}
.picker-enter-from,
.picker-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* Workspace Apps */
.workspace-toggle {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  background: var(--paper);
  border: 2px solid var(--rule-line);
  cursor: pointer;
  transition: all 0.15s ease;
  color: var(--ink);
  font-family: var(--font-body);
  font-size: 0.9rem;
  text-align: left;
}

.workspace-toggle:hover {
  border-color: var(--ink);
}

.workspace-toggle .chevron {
  margin-left: auto;
}

.workspace-apps-panel {
  background: var(--paper);
  border: 2px solid var(--ink);
  box-shadow: 4px 4px 0 var(--ink-shadow);
  margin-top: 4px;
  padding: 16px;
}

.workspace-hint {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--ink-light);
  margin: 0 0 16px 0;
  line-height: 1.5;
}

.workspace-app-entry {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.app-fields {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.app-input {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--ink);
  background: var(--paper-alt);
  border: 1px solid var(--rule-line);
  padding: 8px 10px;
  transition: all 0.15s ease;
  width: 100%;
}

.app-input::placeholder {
  color: var(--ink-light);
  opacity: 0.7;
}

.app-input:focus {
  outline: none;
  border-color: var(--ink);
}

.app-input--command,
.app-input--dir {
  font-family: var(--font-mono);
  font-size: 0.85rem;
}

.app-remove-btn {
  width: 32px;
  height: 32px;
  background: transparent;
  border: 1px solid transparent;
  color: var(--ink-light);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  flex-shrink: 0;
  align-self: center;
}

.app-remove-btn:hover {
  color: var(--error);
  border-color: var(--error);
}

.add-app-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 10px;
  font-family: var(--font-mono);
  font-size: 0.8rem;
  color: var(--ink-light);
  background: transparent;
  border: 2px dashed var(--rule-line);
  cursor: pointer;
  transition: all 0.15s ease;
}

.add-app-btn:hover {
  color: var(--ink);
  border-color: var(--ink);
  background: var(--paper-alt);
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

  .color-picker {
    grid-template-columns: repeat(11, 1fr);
    gap: 2px;
    padding: 6px;
  }
}
</style>
