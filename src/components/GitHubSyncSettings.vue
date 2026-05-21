<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { GitHubSyncConfig } from "../types/github-sync";

const props = defineProps<{ modelValue: GitHubSyncConfig }>();
const emit = defineEmits<{
  "update:modelValue": [config: GitHubSyncConfig];
  close: [];
  saved: [];
}>();

function normalizeRepo(value: string): string {
  return value
    .trim()
    .replace(/^https?:\/\/github\.com\//, "")
    .replace(/\.git$/, "")
    .replace(/\/$/, "");
}

function normalizedConfig(c: GitHubSyncConfig): GitHubSyncConfig {
  return { ...c, repo: normalizeRepo(c.repo) };
}

const form = ref<GitHubSyncConfig>(normalizedConfig(props.modelValue));
watch(() => props.modelValue, (v) => { form.value = normalizedConfig(v); });

const showToken = ref(false);

const testStatus = ref<"idle" | "testing" | "ok" | "error">("idle");
const testMessage = ref("");
const saveStatus = ref<"idle" | "saving" | "saved" | "error">("idle");

const POLL_OPTIONS = [
  { label: "1 minute", value: 60 },
  { label: "2 minutes", value: 120 },
  { label: "5 minutes", value: 300 },
  { label: "Manual only", value: 0 },
];

async function testConnection() {
  if (!form.value.token || !form.value.repo) {
    testStatus.value = "error";
    testMessage.value = "Token and repository are required.";
    return;
  }
  testStatus.value = "testing";
  testMessage.value = "";
  try {
    const count = await invoke<number>("test_github_connection", { config: form.value });
    testStatus.value = "ok";
    testMessage.value = `Connected! Found ${count} todo${count !== 1 ? "s" : ""} on GitHub.`;
  } catch (e) {
    testStatus.value = "error";
    testMessage.value = String(e);
  }
}

async function save(enable: boolean) {
  saveStatus.value = "saving";
  try {
    const updated: GitHubSyncConfig = { ...form.value, enabled: enable };
    await invoke("save_github_sync_config", { config: updated });
    form.value = updated;
    emit("update:modelValue", updated);
    saveStatus.value = "saved";
    emit("saved");
    setTimeout(() => { saveStatus.value = "idle"; }, 2000);
  } catch (e) {
    saveStatus.value = "error";
    setTimeout(() => { saveStatus.value = "idle"; }, 3000);
  }
}
</script>

<template>
  <div class="sync-settings">
    <div class="sync-settings__intro">
      <p>
        Store your <code>todos.json</code> in a private GitHub repository for automatic
        backup and multi-device sync.
      </p>
      <p class="sync-settings__note">
        Your data stays in your private repo. Tick Later reads and writes it directly
        using the GitHub API — no third-party server involved.
      </p>
    </div>

    <!-- Repository -->
    <div class="field">
      <label class="field__label">Repository</label>
      <input
        :value="form.repo"
        @input="form.repo = normalizeRepo(($event.target as HTMLInputElement).value)"
        type="text"
        class="field__input"
        placeholder="your-username/private-todos"
      />
      <span class="field__hint">Paste the GitHub URL or enter <code>owner/repo</code> directly.</span>
    </div>

    <!-- Personal Access Token -->
    <div class="field">
      <label class="field__label">
        Personal Access Token
        <a
          href="https://github.com/settings/personal-access-tokens/new"
          class="field__hint-link"
          target="_blank"
          rel="noopener"
        >Create fine-grained token ↗</a>
      </label>
      <div class="token-wrapper">
        <input
          v-model="form.token"
          :type="showToken ? 'text' : 'password'"
          class="field__input"
          placeholder="github_pat_…"
          autocomplete="off"
        />
        <button type="button" class="token-toggle" @click="showToken = !showToken" :title="showToken ? 'Hide token' : 'Show token'">
          <!-- eye-off -->
          <svg v-if="showToken" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
            <line x1="1" y1="1" x2="23" y2="23"/>
          </svg>
          <!-- eye -->
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
            <circle cx="12" cy="12" r="3"/>
          </svg>
        </button>
      </div>
      <span class="field__hint">
        Use a <strong>fine-grained token</strong> and restrict it to your repo given above only.
        Set Repository permissions → <code>Contents: Read &amp; Write</code>. No other permissions needed.
      </span>
    </div>

    <!-- File path -->
    <div class="field">
      <label class="field__label">File path</label>
      <input
        v-model="form.file_path"
        type="text"
        class="field__input"
        placeholder="todos.json"
      />
      <span class="field__hint">Path inside the repo where todos are stored.</span>
    </div>

    <!-- Poll interval -->
    <div class="field">
      <label class="field__label">Background sync interval</label>
      <div class="poll-options">
        <label
          v-for="opt in POLL_OPTIONS"
          :key="opt.value"
          class="poll-option"
          :class="{ 'poll-option--active': form.poll_interval_secs === opt.value }"
        >
          <input
            type="radio"
            :value="opt.value"
            v-model="form.poll_interval_secs"
            class="poll-option__radio"
          />
          {{ opt.label }}
        </label>
      </div>
    </div>

    <!-- Test connection -->
    <div class="test-row">
      <button class="btn btn--ghost" @click="testConnection" :disabled="testStatus === 'testing'">
        <svg v-if="testStatus !== 'testing'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>
        </svg>
        <svg v-else class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
        </svg>
        Test connection
      </button>
      <span
        v-if="testStatus !== 'idle' && testStatus !== 'testing'"
        class="test-message"
        :class="testStatus === 'ok' ? 'test-message--ok' : 'test-message--error'"
      >{{ testMessage }}</span>
    </div>

    <!-- Actions -->
    <div class="actions">
      <button
        class="btn btn--primary"
        @click="save(true)"
        :disabled="saveStatus === 'saving'"
      >
        {{ saveStatus === 'saved' ? 'Saved!' : 'Save & Enable' }}
      </button>
      <button
        v-if="modelValue.enabled"
        class="btn btn--ghost btn--danger"
        @click="save(false)"
        :disabled="saveStatus === 'saving'"
      >
        Disable sync
      </button>
      <button class="btn btn--ghost" @click="emit('close')">Cancel</button>
    </div>
  </div>
</template>

<style scoped>
.sync-settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 20px 28px 28px;
}

.sync-settings__intro {
  font-size: 0.9rem;
  color: var(--ink-light);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sync-settings__note {
  font-size: 0.82rem;
  border-left: 3px solid var(--rule-line);
  padding-left: 10px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field__label {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--ink);
  display: flex;
  align-items: center;
  gap: 10px;
}

.field__hint-link {
  font-size: 0.65rem;
  color: var(--accent);
  text-decoration: none;
  font-weight: 400;
  letter-spacing: 0.04em;
}
.field__hint-link:hover { text-decoration: underline; }

.token-wrapper {
  position: relative;
  display: flex;
  align-items: stretch;
}

.token-wrapper .field__input {
  flex: 1;
  padding-right: 40px;
}

.token-toggle {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 38px;
  background: transparent;
  border: none;
  color: var(--ink-light);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.15s;
}
.token-toggle:hover { color: var(--ink); }

.field__input {
  font-family: var(--font-mono);
  font-size: 0.85rem;
  padding: 8px 12px;
  background: var(--paper);
  border: 2px solid var(--rule-line);
  color: var(--ink);
  outline: none;
  width: 100%;
}
.field__input:focus { border-color: var(--ink); }

.field__hint {
  font-size: 0.78rem;
  color: var(--ink-light);
}
.field__hint code {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  background: var(--paper-alt);
  padding: 1px 4px;
  border: 1px solid var(--rule-line);
}

.poll-options {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.poll-option {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  font-family: var(--font-mono);
  font-size: 0.72rem;
  font-weight: 500;
  letter-spacing: 0.04em;
  border: 2px solid var(--rule-line);
  cursor: pointer;
  transition: border-color 0.1s, background 0.1s;
  color: var(--ink-light);
}
.poll-option--active {
  border-color: var(--ink);
  color: var(--ink);
  background: var(--paper-alt);
}
.poll-option__radio { display: none; }

.test-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.test-message {
  font-size: 0.82rem;
  font-family: var(--font-body);
}
.test-message--ok { color: var(--success); }
.test-message--error { color: var(--urgent); }

.actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  padding-top: 8px;
  border-top: 1px dashed var(--rule-line);
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 18px;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  border: 2px solid var(--ink);
  cursor: pointer;
  transition: transform 0.1s, box-shadow 0.1s;
  background: transparent;
  color: var(--ink);
}
.btn:hover:not(:disabled) { transform: translate(-2px,-2px); box-shadow: 3px 3px 0 var(--ink-shadow); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn--primary { background: var(--ink); color: var(--paper); }
.btn--danger { border-color: var(--urgent); color: var(--urgent); }
.btn--danger:hover:not(:disabled) { box-shadow: 3px 3px 0 rgba(185,28,28,0.25); }

.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
