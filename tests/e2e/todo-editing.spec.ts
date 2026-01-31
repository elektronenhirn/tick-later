import { expect } from "@wdio/globals";
import {
  createTempDbPath,
  cleanupTempDb,
  setupTestEnvironment,
  createTodo,
  findTodoByTitle,
} from "./helpers/test-setup";

describe("Todo Editing", () => {
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

  it("should open edit dialog when double-clicking a todo", async () => {
    // Create a todo
    const todoTitle = `Double Click Test ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Find the todo item
    const todoItem = await findTodoByTitle(todoTitle);

    // Double-click to open edit dialog
    await todoItem.doubleClick();

    // Verify the edit modal appears
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Verify it's in edit mode (header says "Edit Entry")
    const modalHeader = await $(".modal-header h2");
    const headerText = await modalHeader.getText();
    expect(headerText).toBe("Edit Entry");

    // Verify the title is pre-filled
    const titleInput = await $("#title");
    const titleValue = await titleInput.getValue();
    expect(titleValue).toBe(todoTitle);

    // Close the modal
    const closeBtn = await $(".close-btn");
    await closeBtn.click();
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    console.log(`Successfully opened edit dialog for "${todoTitle}"`);
  });

  it("should change the todo title", async () => {
    // Create a todo
    const originalTitle = `Original Title ${Date.now()}`;
    await createTodo({ title: originalTitle });

    // Find and double-click the todo to edit
    const todoItem = await findTodoByTitle(originalTitle);
    await todoItem.doubleClick();

    // Wait for edit modal
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Clear and set new title
    const newTitle = `Updated Title ${Date.now()}`;
    const titleInput = await $("#title");
    await titleInput.clearValue();
    await titleInput.setValue(newTitle);

    // Submit the form
    const submitBtn = await $(".submit-btn");
    await submitBtn.click();

    // Wait for modal to close
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Verify the old title is gone and new title exists
    const oldTodoItem = await $(`article.todo-item*=${originalTitle}`);
    expect(await oldTodoItem.isExisting()).toBe(false);

    const newTodoItem = await $(`article.todo-item*=${newTitle}`);
    expect(await newTodoItem.isExisting()).toBe(true);

    console.log(`Successfully changed title from "${originalTitle}" to "${newTitle}"`);
  });

  it("should change the todo description", async () => {
    // Create a todo without description
    const todoTitle = `Description Test ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Find and double-click the todo to edit
    const todoItem = await findTodoByTitle(todoTitle);
    await todoItem.doubleClick();

    // Wait for edit modal
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Add a description
    const newDescription = `Added description ${Date.now()}`;
    const descInput = await $("#description");
    await descInput.setValue(newDescription);

    // Submit the form
    const submitBtn = await $(".submit-btn");
    await submitBtn.click();

    // Wait for modal to close
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Find the todo and verify description appears
    const updatedTodo = await findTodoByTitle(todoTitle);
    const descriptionElement = await updatedTodo.$(".item-description");
    await descriptionElement.waitForDisplayed({ timeout: 5000 });

    const descriptionText = await descriptionElement.getText();
    expect(descriptionText).toContain(newDescription);

    console.log(`Successfully added description to "${todoTitle}"`);
  });

  it("should change the todo color", async () => {
    // Create a todo
    const todoTitle = `Color Test ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Find and double-click the todo to edit
    const todoItem = await findTodoByTitle(todoTitle);

    // Scroll to the todo first
    await todoItem.scrollIntoView();
    await browser.pause(300);

    // Get the initial sidepanel color style
    const sidepanelBefore = await todoItem.$(".sidepanel");
    const styleBefore = await sidepanelBefore.getAttribute("style");

    // Double-click to edit
    await todoItem.doubleClick();

    // Wait for edit modal
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Scroll modal into view
    await modal.scrollIntoView();
    await browser.pause(200);

    // Open color picker
    const colorPreview = await $(".color-preview");
    await colorPreview.scrollIntoView();
    await colorPreview.click();

    // Wait for color picker to appear
    const colorPicker = await $(".color-picker");
    await colorPicker.waitForDisplayed({ timeout: 3000 });

    // Select a specific color using JavaScript to avoid scroll issues
    await browser.execute(() => {
      const colorOptions = document.querySelectorAll(".color-option:not(.color-option--random)");
      if (colorOptions.length > 4) {
        (colorOptions[4] as HTMLElement).click();
      } else if (colorOptions.length > 0) {
        (colorOptions[0] as HTMLElement).click();
      }
    });

    await browser.pause(300);

    // Submit the form
    const submitBtn = await $(".submit-btn");
    await submitBtn.scrollIntoView();
    await submitBtn.click();

    // Wait for modal to close
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Find the todo again and verify the color changed
    const updatedTodo = await findTodoByTitle(todoTitle);
    const sidepanelAfter = await updatedTodo.$(".sidepanel");
    const styleAfter = await sidepanelAfter.getAttribute("style");

    // The style should be different (new color applied)
    expect(styleAfter).not.toBe(styleBefore);

    console.log(`Successfully changed color for "${todoTitle}"`);
  });

  it("should adjust the due date manually in the edit dialog", async () => {
    // Create a todo
    const todoTitle = `Due Date Test ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Find and double-click the todo to edit
    const todoItem = await findTodoByTitle(todoTitle);

    // Scroll to the todo first
    await todoItem.scrollIntoView();
    await browser.pause(300);

    // Get the initial due date display
    const scheduleTimeBefore = await todoItem.$(".schedule-time");
    const dueDateBefore = await scheduleTimeBefore.getText();

    // Double-click to edit
    await todoItem.doubleClick();

    // Wait for edit modal
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Scroll modal into view
    await modal.scrollIntoView();
    await browser.pause(200);

    // Change the due date using the "Tomorrow 9AM" preset button
    // (Using datetime-local input directly can be unreliable in WebDriver)
    const tomorrowBtn = await $(
      `//button[contains(@class, "preset-btn") and contains(., "Tomorrow")]`
    );
    await tomorrowBtn.scrollIntoView();
    await tomorrowBtn.click();

    // Wait for the value to be set
    await browser.pause(300);

    // Submit the form
    const submitBtn = await $(".submit-btn");
    await submitBtn.scrollIntoView();
    await submitBtn.waitForEnabled({ timeout: 5000 });
    await submitBtn.click();

    // Wait for modal to close
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Find the todo again and verify the due date changed
    const updatedTodo = await findTodoByTitle(todoTitle);
    const scheduleTimeAfter = await updatedTodo.$(".schedule-time");
    const dueDateAfter = await scheduleTimeAfter.getText();

    // The due date should be different
    expect(dueDateAfter).not.toBe(dueDateBefore);
    // Should contain "9:00" (from Tomorrow 9AM preset)
    expect(dueDateAfter).toContain("9:00");

    console.log(
      `Successfully changed due date for "${todoTitle}" from "${dueDateBefore}" to "${dueDateAfter}"`
    );
  });

  it("should open edit dialog via edit button", async () => {
    // Create a todo
    const todoTitle = `Edit Button Test ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Find the todo item
    const todoItem = await findTodoByTitle(todoTitle);

    // Scroll to the todo and hover to reveal action buttons
    await todoItem.scrollIntoView();
    await todoItem.moveTo();
    await browser.pause(300);

    // Click the edit button
    const editBtn = await todoItem.$(".action-btn--edit");
    await editBtn.waitForDisplayed({ timeout: 5000 });
    await editBtn.click();

    // Verify the edit modal appears
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Verify it's in edit mode
    const modalHeader = await $(".modal-header h2");
    const headerText = await modalHeader.getText();
    expect(headerText).toBe("Edit Entry");

    // Close the modal
    const closeBtn = await $(".close-btn");
    await closeBtn.click();
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    console.log(`Successfully opened edit dialog via edit button for "${todoTitle}"`);
  });
});
