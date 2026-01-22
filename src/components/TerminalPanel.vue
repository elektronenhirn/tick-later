<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import "@xterm/xterm/css/xterm.css";
import type { TerminalOutputEvent, TerminalExitedEvent } from "../types/terminal";

const props = defineProps<{
  todoId: string | null;
  todoTitle: string | null;
  isVisible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const terminalContainer = ref<HTMLElement | null>(null);

// Store terminal instances per todo
interface TerminalInstance {
  terminal: Terminal;
  fitAddon: FitAddon;
  element: HTMLDivElement;
}
const terminals = new Map<string, TerminalInstance>();
const activeSessions = new Set<string>();

let unlistenOutputFn: UnlistenFn | null = null;
let unlistenExitFn: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;
let currentTodoId: string | null = null;

function getTerminalTheme() {
  const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  return isDark
    ? {
        background: "#1c1917",
        foreground: "#fafaf9",
        cursor: "#fafaf9",
        cursorAccent: "#1c1917",
        selectionBackground: "#44403c",
      }
    : {
        background: "#faf8f3",
        foreground: "#1a1614",
        cursor: "#1a1614",
        cursorAccent: "#faf8f3",
        selectionBackground: "#d4cfc5",
      };
}

function createTerminalInstance(todoId: string): TerminalInstance {
  const terminal = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: "'JetBrains Mono', 'SF Mono', Consolas, monospace",
    scrollback: 10000,
    smoothScrollDuration: 100,
    theme: getTerminalTheme(),
  });

  const fitAddon = new FitAddon();
  const webLinksAddon = new WebLinksAddon();

  terminal.loadAddon(fitAddon);
  terminal.loadAddon(webLinksAddon);

  // Create a container element for this terminal
  const element = document.createElement("div");
  element.className = "terminal-instance";
  element.style.display = "none";
  element.style.width = "100%";
  element.style.height = "100%";

  if (terminalContainer.value) {
    terminalContainer.value.appendChild(element);
  }

  terminal.open(element);

  // Handle user input for this terminal
  terminal.onData(async (data) => {
    if (activeSessions.has(todoId)) {
      try {
        await invoke("write_to_terminal", {
          todoId: todoId,
          data: data,
        });
      } catch (e) {
        console.log("Write to terminal failed:", e);
      }
    }
  });

  return { terminal, fitAddon, element };
}

function showTerminal(todoId: string) {
  // Hide all terminals
  terminals.forEach((instance) => {
    instance.element.style.display = "none";
  });

  // Get or create terminal for this todo
  let instance = terminals.get(todoId);
  if (!instance) {
    instance = createTerminalInstance(todoId);
    terminals.set(todoId, instance);
  }

  // Show and fit this terminal
  instance.element.style.display = "block";
  currentTodoId = todoId;

  nextTick(() => {
    instance!.fitAddon.fit();
    instance!.terminal.focus();
  });
}

async function createSession(todoId: string) {
  if (activeSessions.has(todoId)) {
    return;
  }

  try {
    await invoke("create_terminal_session", {
      todoId: todoId,
      workingDir: null,
    });
    activeSessions.add(todoId);

    // Resize after session is created
    const instance = terminals.get(todoId);
    if (instance) {
      await nextTick();
      instance.fitAddon.fit();
      await invoke("resize_terminal", {
        todoId: todoId,
        rows: instance.terminal.rows,
        cols: instance.terminal.cols,
      });
    }
  } catch (e) {
    console.error("Failed to create terminal session:", e);
  }
}

async function handleSessionExit(todoId: string) {
  console.log("Terminal exited for:", todoId);

  // Mark session as inactive
  activeSessions.delete(todoId);

  // Close the old session on the backend
  try {
    await invoke("close_terminal_session", { todoId: todoId });
  } catch (e) {
    // Ignore - session might already be closed
  }

  // Clean up the terminal instance
  const instance = terminals.get(todoId);
  if (instance) {
    instance.terminal.dispose();
    instance.element.remove();
    terminals.delete(todoId);
  }

  // Close the panel if this was the active terminal
  if (currentTodoId === todoId) {
    emit("close");
  }
}

async function initTerminalSystem() {
  if (!terminalContainer.value) return;

  // Setup resize observer for the container
  resizeObserver = new ResizeObserver(() => {
    if (props.isVisible && currentTodoId) {
      const instance = terminals.get(currentTodoId);
      if (instance) {
        instance.fitAddon.fit();
        if (activeSessions.has(currentTodoId)) {
          invoke("resize_terminal", {
            todoId: currentTodoId,
            rows: instance.terminal.rows,
            cols: instance.terminal.cols,
          }).catch(console.error);
        }
      }
    }
  });
  resizeObserver.observe(terminalContainer.value);

  // Listen for terminal output events - route to correct terminal
  unlistenOutputFn = await listen<TerminalOutputEvent>("terminal-output", (event) => {
    const instance = terminals.get(event.payload.todo_id);
    if (instance) {
      instance.terminal.write(event.payload.data);
    }
  });

  // Listen for terminal exit events - close the panel
  unlistenExitFn = await listen<TerminalExitedEvent>("terminal-exited", (event) => {
    console.log("Received terminal-exited event:", event.payload);
    handleSessionExit(event.payload.todo_id);
  });
}

// Watch for todoId changes to switch sessions
watch(
  () => props.todoId,
  async (newId) => {
    if (newId && props.isVisible) {
      showTerminal(newId);
      if (!activeSessions.has(newId)) {
        await createSession(newId);
      }
    }
  }
);

// Watch for visibility changes
watch(
  () => props.isVisible,
  async (visible) => {
    if (visible && props.todoId) {
      await nextTick();
      showTerminal(props.todoId);
      if (!activeSessions.has(props.todoId)) {
        await createSession(props.todoId);
      }
    }
  }
);

onMounted(async () => {
  await initTerminalSystem();
  if (props.isVisible && props.todoId) {
    showTerminal(props.todoId);
    await createSession(props.todoId);
  }
});

onUnmounted(() => {
  if (unlistenOutputFn) {
    unlistenOutputFn();
  }
  if (unlistenExitFn) {
    unlistenExitFn();
  }
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  // Dispose all terminal instances
  terminals.forEach((instance) => {
    instance.terminal.dispose();
    instance.element.remove();
  });
  terminals.clear();
});
</script>

<template>
  <aside class="terminal-panel" :class="{ visible: isVisible }">
    <header class="terminal-header">
      <div class="terminal-info">
        <span class="terminal-badge">TERMINAL</span>
        <span class="terminal-todo" v-if="todoTitle">{{ todoTitle }}</span>
      </div>
      <button @click="emit('close')" class="terminal-close-btn" title="Close terminal">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
      </button>
    </header>
    <div ref="terminalContainer" class="terminal-container"></div>
  </aside>
</template>

<style scoped>
.terminal-panel {
  width: 0;
  height: 100%;
  background: var(--paper-alt);
  border-left: 2px solid var(--rule-line);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.terminal-panel.visible {
  width: 50%;
}

.terminal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 2px solid var(--rule-line);
  background: var(--paper);
  flex-shrink: 0;
}

.terminal-info {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.terminal-badge {
  font-family: var(--font-mono);
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  color: var(--ink);
  padding: 4px 8px;
  border: 2px solid var(--ink);
  flex-shrink: 0;
}

.terminal-todo {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--ink-light);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.terminal-close-btn {
  width: 32px;
  height: 32px;
  background: transparent;
  border: 2px solid transparent;
  color: var(--ink-light);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.terminal-close-btn:hover {
  color: var(--ink);
  border-color: var(--ink);
}

.terminal-container {
  flex: 1;
  padding: 8px;
  min-height: 0; /* Important for flexbox to allow shrinking */
}

/* Override xterm default styles to fit container */
.terminal-container :deep(.xterm) {
  height: 100%;
}

.terminal-container :deep(.xterm-viewport) {
  overflow-y: scroll !important;
}

.terminal-container :deep(.xterm-screen) {
  height: 100%;
}

@media (max-width: 900px) {
  .terminal-panel.visible {
    width: 100%;
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    z-index: 100;
  }
}
</style>
