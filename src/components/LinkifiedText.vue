<script setup lang="ts">
import { computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps<{
  text: string;
  virtualDesktop?: number;
}>();

// URL regex that matches http, https, and www URLs
const urlRegex = /(https?:\/\/[^\s<>[\]{}|\\^`"]+|www\.[^\s<>[\]{}|\\^`"]+)/gi;

interface TextPart {
  type: "text" | "url";
  content: string;
  href?: string;
}

const parts = computed<TextPart[]>(() => {
  const result: TextPart[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  // Reset regex lastIndex
  urlRegex.lastIndex = 0;

  while ((match = urlRegex.exec(props.text)) !== null) {
    // Add text before the URL
    if (match.index > lastIndex) {
      result.push({
        type: "text",
        content: props.text.slice(lastIndex, match.index),
      });
    }

    // Add the URL
    const url = match[0];
    const href = url.startsWith("www.") ? `https://${url}` : url;
    result.push({
      type: "url",
      content: url,
      href,
    });

    lastIndex = match.index + match[0].length;
  }

  // Add remaining text after last URL
  if (lastIndex < props.text.length) {
    result.push({
      type: "text",
      content: props.text.slice(lastIndex),
    });
  }

  return result;
});

async function handleLinkClick(event: MouseEvent, href: string) {
  event.preventDefault();
  event.stopPropagation();
  try {
    // If a virtual desktop is associated, use the special command that
    // opens the URL and moves the browser window to that desktop
    if (props.virtualDesktop !== undefined) {
      try {
        await invoke("open_url_on_desktop", {
          url: href,
          desktop: props.virtualDesktop
        });
        return;
      } catch (error) {
        console.warn("Failed to open URL on desktop:", error);
        // Fall through to regular URL opening
      }
    }
    await openUrl(href);
  } catch (error) {
    console.error("Failed to open URL:", error);
  }
}
</script>

<template>
  <span class="linkified-text">
    <template v-for="(part, index) in parts" :key="index">
      <a
        v-if="part.type === 'url'"
        class="linkified-link"
        :href="part.href"
        @click="handleLinkClick($event, part.href!)"
        :title="part.href"
      >{{ part.content }}</a>
      <template v-else>{{ part.content }}</template>
    </template>
  </span>
</template>

<style scoped>
.linkified-text {
  display: inline;
}

.linkified-link {
  color: var(--accent);
  text-decoration: underline;
  text-decoration-style: dotted;
  text-underline-offset: 2px;
  cursor: pointer;
  transition: color 0.15s ease;
}

.linkified-link:hover {
  color: var(--ink);
  text-decoration-style: solid;
}
</style>
