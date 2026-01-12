# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Frontend Development
- `npm run dev` - Start development server with hot reload on port 1420
- `npm run build` - Build Vue frontend for production (includes TypeScript compilation)
- `npm run preview` - Preview production build

### Tauri Development
- `npm run tauri dev` - Start Tauri development mode (runs both frontend and backend)
- `npm run tauri build` - Build production Tauri application for current platform

### TypeScript
- `vue-tsc --noEmit` - Type check TypeScript without emitting files (runs as part of build)

## Architecture Overview

This is a **Tauri desktop application** with a **Vue.js 3 + TypeScript frontend**. The architecture follows Tauri's hybrid approach:

### Frontend (src/)
- **Vue 3** with Composition API (`<script setup>`)
- **TypeScript** for type safety
- **Vite** as build tool and development server
- **Single Page Application** mounted at `#app`

### Backend (src-tauri/)
- **Rust** backend using Tauri framework
- **Library structure**: Core logic in `lib.rs`, entry point in `main.rs`
- **Tauri commands**: Functions exposed to frontend via `#[tauri::command]`
- **Plugins**: Currently uses `tauri-plugin-opener`

### Inter-Process Communication
- Frontend invokes Rust functions using `invoke()` from `@tauri-apps/api/core`
- Commands must be registered in `invoke_handler` in `lib.rs`
- Example: `greet` command takes a string parameter and returns formatted greeting

### Key Configuration
- **tauri.conf.json**: App metadata, window settings, build configuration
- **Cargo.toml**: Rust dependencies and library configuration (lib name: `tick_later_lib`)
- **vite.config.ts**: Frontend build config optimized for Tauri (fixed port 1420, ignores `src-tauri`)
- **package.json**: Frontend dependencies and npm scripts

### Development Flow
1. Run `npm run tauri dev` to start both frontend (Vite) and backend (Rust) in development mode
2. Frontend communicates with backend through Tauri's IPC system
3. Hot reload works for Vue components; Rust changes require restart