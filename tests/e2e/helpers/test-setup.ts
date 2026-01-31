import { tmpdir } from "os";
import { join } from "path";
import { unlinkSync, existsSync } from "fs";

/**
 * Creates a unique temporary database path for E2E tests.
 */
export function createTempDbPath(): string {
  return join(
    tmpdir(),
    `tick-later-e2e-${Date.now()}-${Math.random().toString(36).substring(7)}.json`
  );
}

/**
 * Cleans up a temporary database file.
 */
export function cleanupTempDb(tempDbPath: string): void {
  try {
    if (existsSync(tempDbPath)) {
      unlinkSync(tempDbPath);
      console.log(`Cleaned up temp database: ${tempDbPath}`);
    }
  } catch (err) {
    console.error(`Failed to clean up temp database: ${err}`);
  }
}

/**
 * Switches the app to use the temporary test database.
 * Call this in beforeEach after the app has loaded.
 */
export async function switchToTempDatabase(tempDbPath: string): Promise<void> {
  await browser.execute(async (path: string) => {
    // @ts-ignore - Tauri API is available in the app context
    await window.__TAURI__.core.invoke("switch_database", { path });
    // Reload todos after switching database
    // @ts-ignore
    const todos = await window.__TAURI__.core.invoke("load_todos");
    // Dispatch a custom event to notify Vue of the database change
    window.dispatchEvent(
      new CustomEvent("e2e-database-switched", { detail: { todos } })
    );
  }, tempDbPath);
  console.log(`Switched to temp database: ${tempDbPath}`);
}

/**
 * Waits for the app to fully load and switches to the temp database.
 * Use this in beforeEach hook.
 */
export async function setupTestEnvironment(tempDbPath: string): Promise<void> {
  // Wait a bit for the app to start
  await browser.pause(3000);

  // Wait for the app to fully load by checking for the app header
  const appHeader = await $(".app-header");
  await appHeader.waitForDisplayed({ timeout: 30000 });

  // Switch to the temporary test database
  await switchToTempDatabase(tempDbPath);

  // Additional wait for Vue to finish rendering
  await browser.pause(1000);
}

/**
 * Helper to create a new todo with the given title.
 * Returns the unique title used.
 */
export async function createTodo(options: {
  title?: string;
  description?: string;
  color?: string;
}): Promise<string> {
  const todoTitle = options.title || `Test Todo ${Date.now()}`;

  // Click the compose button to open the "New Entry" modal
  const composeBtn = await $(".compose-btn");
  await composeBtn.waitForDisplayed({ timeout: 15000 });
  await composeBtn.click();

  // Wait for the modal to appear
  const modal = await $(".modal-content");
  await modal.waitForDisplayed({ timeout: 5000 });

  // Fill in the title
  const titleInput = await $("#title");
  await titleInput.waitForDisplayed();
  await titleInput.setValue(todoTitle);

  // Fill in description if provided
  if (options.description) {
    const descInput = await $("#description");
    await descInput.setValue(options.description);
  }

  // Select color if provided
  if (options.color) {
    const colorPreview = await $(".color-preview");
    await colorPreview.click();
    await browser.pause(300);

    // Find and click the color option
    const colorOption = await $(
      `.color-option[style*="${options.color}"]`
    );
    if (await colorOption.isExisting()) {
      await colorOption.click();
    }
  }

  // Set a revisit time using the "In 1h" quick schedule button
  const quickScheduleBtn = await $(".preset-btn");
  await quickScheduleBtn.click();

  // Submit the form - scroll into view first to avoid click interception
  const submitBtn = await $(".submit-btn");
  await submitBtn.scrollIntoView();
  await submitBtn.waitForEnabled({ timeout: 5000 });
  await submitBtn.click();

  // Wait for the modal to close
  await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

  // Wait for the todo to appear
  const todoItem = await $(`article.todo-item*=${todoTitle}`);
  await todoItem.waitForDisplayed({ timeout: 5000 });

  return todoTitle;
}

/**
 * Helper to find a todo by its title.
 */
export async function findTodoByTitle(title: string): Promise<WebdriverIO.Element> {
  const todoItem = await $(`article.todo-item*=${title}`);
  await todoItem.waitForDisplayed({ timeout: 5000 });
  return todoItem;
}
