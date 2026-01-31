import { expect } from "@wdio/globals";
import { tmpdir } from "os";
import { join } from "path";
import { unlinkSync, existsSync } from "fs";

describe("Todo Completion", () => {
  // Generate a unique temp database path for this test run
  const tempDbPath = join(
    tmpdir(),
    `tick-later-e2e-${Date.now()}-${Math.random().toString(36).substring(7)}.json`
  );

  before(async () => {
    console.log(`Using temporary database: ${tempDbPath}`);
  });

  after(async () => {
    // Clean up the temp database file after all tests
    try {
      if (existsSync(tempDbPath)) {
        unlinkSync(tempDbPath);
        console.log(`Cleaned up temp database: ${tempDbPath}`);
      }
    } catch (err) {
      console.error(`Failed to clean up temp database: ${err}`);
    }
  });

  beforeEach(async () => {
    // Wait a bit for the app to start
    await browser.pause(3000);

    // Debug: Get page source to see what's loaded
    const pageSource = await browser.getPageSource();
    console.log("Page source length:", pageSource.length);
    console.log("Page source preview:", pageSource.substring(0, 500));

    // Wait for the app to fully load by checking for the app header
    const appHeader = await $(".app-header");
    await appHeader.waitForDisplayed({ timeout: 30000 });

    // Switch to the temporary test database
    const dbPath = tempDbPath;
    await browser.execute(async (path) => {
      // @ts-ignore - Tauri API is available in the app context
      await window.__TAURI__.core.invoke("switch_database", { path });
      // Reload todos after switching database
      // @ts-ignore
      const todos = await window.__TAURI__.core.invoke("load_todos");
      // Dispatch a custom event to notify Vue of the database change
      window.dispatchEvent(
        new CustomEvent("e2e-database-switched", { detail: { todos } })
      );
    }, dbPath);

    console.log(`Switched to temp database: ${dbPath}`);

    // Additional wait for Vue to finish rendering
    await browser.pause(1000);
  });

  it("should move a todo to the Completed section when ticked", async () => {
    // Generate a unique title for this test
    const todoTitle = `Test Todo ${Date.now()}`;

    // Step 1: Click the compose button to open the "New Entry" modal
    const composeBtn = await $(".compose-btn");
    await composeBtn.waitForDisplayed({ timeout: 15000 });
    await composeBtn.click();

    // Step 2: Wait for the modal to appear
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Step 3: Fill in the title
    const titleInput = await $("#title");
    await titleInput.waitForDisplayed();
    await titleInput.setValue(todoTitle);

    // Step 4: Set a revisit time using the "In 1h" quick schedule button
    const quickScheduleBtn = await $(".preset-btn");
    await quickScheduleBtn.click();

    // Step 5: Submit the form
    const submitBtn = await $(".submit-btn");
    await submitBtn.waitForEnabled({ timeout: 5000 });
    await submitBtn.click();

    // Step 6: Wait for the modal to close
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Step 7: Find the newly created todo item by its title
    const todoItem = await $(`article.todo-item*=${todoTitle}`);
    await todoItem.waitForDisplayed({ timeout: 5000 });

    // Step 8: Verify the todo is NOT in the completed section yet
    const completedSection = await $(".section--completed");
    const isCompletedSectionDisplayed = await completedSection.isDisplayed();

    if (isCompletedSectionDisplayed) {
      // If completed section exists, verify our todo is not in it
      const todoInCompleted = await completedSection.$(
        `article.todo-item*=${todoTitle}`
      );
      expect(await todoInCompleted.isExisting()).toBe(false);
    }

    // Step 9: Click the checkbox to mark the todo as complete
    const checkbox = await todoItem.$(".checkbox");
    await checkbox.waitForClickable({ timeout: 5000 });
    await checkbox.click();

    // Step 10: Wait for the todo to move to the completed section
    await browser.pause(500); // Small pause for animation/transition

    // Step 11: Verify the completed section now exists and contains our todo
    const completedSectionAfter = await $(".section--completed");
    await completedSectionAfter.waitForDisplayed({ timeout: 5000 });

    const todoInCompletedSection = await completedSectionAfter.$(
      `article.todo-item*=${todoTitle}`
    );
    await todoInCompletedSection.waitForDisplayed({ timeout: 5000 });

    // Step 12: Verify the todo has the completed class
    const todoClasses = await todoInCompletedSection.getAttribute("class");
    expect(todoClasses).toContain("todo-item--completed");

    // Step 13: Verify the checkbox shows the checkmark (has the checked class)
    const checkboxInCompleted = await todoInCompletedSection.$(".checkbox");
    const checkboxClasses = await checkboxInCompleted.getAttribute("class");
    expect(checkboxClasses).toContain("checkbox--checked");

    console.log(
      `Successfully verified todo "${todoTitle}" was moved to Completed section`
    );
  });

  it("should remove a todo from Completed section when unticked", async () => {
    // Generate a unique title for this test
    const todoTitle = `Untick Test ${Date.now()}`;

    // Create a new todo
    const composeBtn = await $(".compose-btn");
    await composeBtn.click();

    const modal = await $(".modal-content");
    await modal.waitForDisplayed();

    const titleInput = await $("#title");
    await titleInput.setValue(todoTitle);

    const quickScheduleBtn = await $(".preset-btn");
    await quickScheduleBtn.click();

    const submitBtn = await $(".submit-btn");
    await submitBtn.waitForEnabled();
    await submitBtn.click();

    await modal.waitForDisplayed({ reverse: true });

    // Find and complete the todo
    const todoItem = await $(`article.todo-item*=${todoTitle}`);
    await todoItem.waitForDisplayed();

    const checkbox = await todoItem.$(".checkbox");
    await checkbox.click();

    // Wait for it to appear in completed section
    await browser.pause(500);
    const completedSection = await $(".section--completed");
    await completedSection.waitForDisplayed();

    const completedTodo = await completedSection.$(
      `article.todo-item*=${todoTitle}`
    );
    await completedTodo.waitForDisplayed();

    // Now untick the todo
    const checkboxInCompleted = await completedTodo.$(".checkbox");
    await checkboxInCompleted.click();

    // Wait for transition
    await browser.pause(500);

    // Verify the todo is no longer in the completed section
    const todoStillInCompleted = await completedSection.$(
      `article.todo-item*=${todoTitle}`
    );
    const isStillInCompleted = await todoStillInCompleted.isExisting();

    if (isStillInCompleted) {
      // If it still exists in DOM, verify it doesn't have the completed class
      const classes = await todoStillInCompleted.getAttribute("class");
      expect(classes).not.toContain("todo-item--completed");
    }

    // Find the todo in the scheduled section (it should be back there)
    // Use XPath for partial text match since CSS *=selector doesn't work with Tauri WebDriver
    const scheduledTodo = await $(
      `//div[contains(@class, "time-grid")]//article[contains(@class, "todo-item") and contains(., "${todoTitle}")]`
    );
    expect(await scheduledTodo.isExisting()).toBe(true);

    console.log(
      `Successfully verified todo "${todoTitle}" was moved back from Completed section`
    );
  });
});
