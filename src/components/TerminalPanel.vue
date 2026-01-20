<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import "@xterm/xterm/css/xterm.css";
import type { TerminalOutputEvent } from "../types/terminal";

const props = defineProps<{
  todoId: string | null;
  todoTitle: string | null;
  isVisible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const terminalContainer = ref<HTMLElement | null>(null);
const isSessionActive = ref(false);

let terminal: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlistenFn: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;

// Track which sessions we've created
const activeSessions = new Set<string>();

async function initTerminal() {
  if (!terminalContainer.value || terminal) return;

  // Check if dark mode is preferred
  const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;

  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: "'JetBrains Mono', 'SF Mono', Consolas, monospace",
    theme: isDark
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
        },
  });

  fitAddon = new FitAddon();
  const webLinksAddon = new WebLinksAddon();

  terminal.loadAddon(fitAddon);
  terminal.loadAddon(webLinksAddon);

  terminal.open(terminalContainer.value);

  // Fit the terminal to the container
  await nextTick();
  fitAddon.fit();

  // Handle user input
  terminal.onData(async (data) => {
    if (props.todoId && isSessionActive.value) {
      try {
        await invoke("write_to_terminal", {
          todoId: props.todoId,
          data: data,
        });
      } catch (e) {
        console.error("Failed to write to terminal:", e);
      }
    }
  });

  // Setup resize observer
  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && terminal && props.isVisible) {
      fitAddon.fit();
      if (props.todoId && isSessionActive.value) {
        invoke("resize_terminal", {
          todoId: props.todoId,
          rows: terminal.rows,
          cols: terminal.cols,
        }).catch(console.error);
      }
    }
  });
  resizeObserver.observe(terminalContainer.value);

  // Listen for terminal output events
  unlistenFn = await listen<TerminalOutputEvent>("terminal-output", (event) => {
    if (event.payload.todo_id === props.todoId && terminal) {
      terminal.write(event.payload.data);
    }
  });
}

async function createSession() {
  if (!props.todoId || activeSessions.has(props.todoId)) {
    isSessionActive.value = activeSessions.has(props.todoId || "");
    return;
  }

  try {
    await invoke("create_terminal_session", {
      todoId: props.todoId,
      workingDir: null,
    });
    activeSessions.add(props.todoId);
    isSessionActive.value = true;

    // Resize after session is created
    if (terminal && fitAddon) {
      await nextTick();
      fitAddon.fit();
      await invoke("resize_terminal", {
        todoId: props.todoId,
        rows: terminal.rows,
        cols: terminal.cols,
      });
    }
  } catch (e) {
    console.error("Failed to create terminal session:", e);
  }
}

function clearTerminal() {
  if (terminal) {
    terminal.clear();
  }
}

// Watch for todoId changes to switch sessions
watch(
  () => props.todoId,
  async (newId, oldId) => {
    if (newId && newId !== oldId) {
      // Clear the terminal display when switching
      clearTerminal();

      // Check if we need to create a new session
      if (!activeSessions.has(newId)) {
        await createSession();
      } else {
        isSessionActive.value = true;
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
      if (fitAddon && terminal) {
        fitAddon.fit();
      }
      if (!activeSessions.has(props.todoId)) {
        await createSession();
      }
      // Focus the terminal when visible
      terminal?.focus();
    }
  }
);

onMounted(async () => {
  await initTerminal();
  if (props.isVisible && props.todoId) {
    await createSession();
  }
});

onUnmounted(() => {
  if (unlistenFn) {
    unlistenFn();
  }
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  if (terminal) {
    terminal.dispose();
  }
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
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: 500px;
  background: var(--paper-alt);
  border-left: 2px solid var(--rule-line);
  display: flex;
  flex-direction: column;
  z-index: 100;
  transform: translateX(100%);
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.terminal-panel.visible {
  transform: translateX(0);
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
  overflow: hidden;
}

/* Override xterm default styles to fit container */
.terminal-container :deep(.xterm) {
  height: 100%;
}

.terminal-container :deep(.xterm-viewport) {
  overflow-y: auto !important;
}

@media (max-width: 1024px) {
  .terminal-panel {
    width: 100%;
  }
}
</style>
