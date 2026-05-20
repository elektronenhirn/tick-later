#!/usr/bin/env node
// Updates revisit_at timestamps in demo.json so that dates which were
// "today" relative to the original reference date become today's date.
//
// Entries with `_demo_relative_hours` have their revisit_at set to
// exactNow + N hours (so they reliably land in the right time bucket).
//
// Entries without that field are day-shifted from their `_base_revisit_at`
// (falling back to `revisit_at` if absent). Using _base_revisit_at as the
// anchor prevents accumulated offset drift when the script is run on
// multiple different days.
//
// The reference "today" in demo.json is 2026-01-25.

import { readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const DEMO_FILE = join(__dirname, '..', 'demo.json');
const REFERENCE_DATE = new Date('2026-01-25T00:00:00Z');

const todos = JSON.parse(readFileSync(DEMO_FILE, 'utf8'));

const exactNow = new Date();
const today = new Date(exactNow);
today.setUTCHours(0, 0, 0, 0);

const dayOffsetMs = today.getTime() - REFERENCE_DATE.getTime();
const dayOffset = Math.round(dayOffsetMs / 86400000);

for (const todo of todos) {
  if (todo._demo_relative_hours !== undefined) {
    const relativeMs = todo._demo_relative_hours * 60 * 60 * 1000;
    todo.revisit_at = new Date(exactNow.getTime() + relativeMs).toISOString();
  } else {
    const base = new Date(todo._base_revisit_at || todo.revisit_at);
    const updated = new Date(base.getTime() + dayOffsetMs);
    todo.revisit_at = updated.toISOString();
  }
}

writeFileSync(DEMO_FILE, JSON.stringify(todos, null, 2) + '\n');
console.log(`Updated ${todos.length} todos in demo.json (shifted by ${dayOffset} days)`);
