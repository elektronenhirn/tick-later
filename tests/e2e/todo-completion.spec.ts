import { expect } from "@wdio/globals";
import {
  createTempDbPath,
  cleanupTempDb,
  setupTestEnvironment,
  createTodo,
  findTodoByTitle,
} from "./helpers/test-setup";

describe("Todo Completion", () => {
  const tempDbPath = createTempDbPath();

  before(async () => {
    console.log(`Using temporary database: ${tempDbPath}`);
  });

  after(async () => {
    cleanupTempDb(tempDbPath);
  });

  beforeEach(async () => {
    await setupTestEnvironment(tempDbPath);
  });

  it("should move a todo to the Completed section when ticked", async () => {
    // Create a todo
    const todoTitle = await createTodo({ title: `Test Todo ${Date.now()}` });

    // Find the todo item
    const todoItem = await findTodoByTitle(todoTitle);

    // Verify the todo is NOT in the completed section yet
    const completedSection = await $(".section--completed");
    const isCompletedSectionDisplayed = await completedSection.isDisplayed();

    if (isCompletedSectionDisplayed) {
      // If completed section exists, verify our todo is not in it
      const todoInCompleted = await completedSection.$(
        `article.todo-item*=${todoTitle}`
      );
      expect(await todoInCompleted.isExisting()).toBe(false);
    }

    // Click the checkbox to mark the todo as complete
    const checkbox = await todoItem.$(".checkbox");
    await checkbox.waitForClickable({ timeout: 5000 });
    await checkbox.click();

    // Wait for the todo to move to the completed section
    await browser.pause(500);

    // Verify the completed section now exists and contains our todo
    const completedSectionAfter = await $(".section--completed");
    await completedSectionAfter.waitForDisplayed({ timeout: 5000 });

    const todoInCompletedSection = await completedSectionAfter.$(
      `article.todo-item*=${todoTitle}`
    );
    await todoInCompletedSection.waitForDisplayed({ timeout: 5000 });

    // Verify the todo has the completed class
    const todoClasses = await todoInCompletedSection.getAttribute("class");
    expect(todoClasses).toContain("todo-item--completed");

    // Verify the checkbox shows the checkmark (has the checked class)
    const checkboxInCompleted = await todoInCompletedSection.$(".checkbox");
    const checkboxClasses = await checkboxInCompleted.getAttribute("class");
    expect(checkboxClasses).toContain("checkbox--checked");

    console.log(
      `Successfully verified todo "${todoTitle}" was moved to Completed section`
    );
  });

  it("should remove a todo from Completed section when unticked", async () => {
    // Create a todo
    const todoTitle = await createTodo({ title: `Untick Test ${Date.now()}` });

    // Find and complete the todo
    const todoItem = await findTodoByTitle(todoTitle);
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
