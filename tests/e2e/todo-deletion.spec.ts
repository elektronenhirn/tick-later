import { expect } from "@wdio/globals";
import {
  createTempDbPath,
  cleanupTempDb,
  setupTestEnvironment,
  createTodo,
  findTodoByTitle,
} from "./helpers/test-setup";

describe("Todo Deletion", () => {
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

  it("should delete a todo when clicking the delete button and confirming", async () => {
    // Create a todo
    const todoTitle = `Delete Test ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Verify the todo exists
    const todoItem = await findTodoByTitle(todoTitle);
    expect(await todoItem.isDisplayed()).toBe(true);

    // Hover over the todo to reveal action buttons
    await todoItem.moveTo();
    await browser.pause(300);

    // Click the delete button
    const deleteBtn = await todoItem.$(".action-btn--delete");
    await deleteBtn.waitForDisplayed({ timeout: 5000 });

    // Set up dialog handler to accept the confirmation
    await browser.execute(() => {
      // Override confirm to return true (accept)
      window.confirm = () => true;
    });

    await deleteBtn.click();

    // Wait a moment for the deletion to process
    await browser.pause(500);

    // Verify the todo no longer exists
    const deletedTodo = await $(`article.todo-item*=${todoTitle}`);
    expect(await deletedTodo.isExisting()).toBe(false);

    console.log(`Successfully deleted todo "${todoTitle}"`);
  });

  it("should NOT delete a todo when canceling the confirmation dialog", async () => {
    // Create a todo
    const todoTitle = `Cancel Delete Test ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Verify the todo exists
    const todoItem = await findTodoByTitle(todoTitle);
    expect(await todoItem.isDisplayed()).toBe(true);

    // Hover over the todo to reveal action buttons
    await todoItem.moveTo();
    await browser.pause(300);

    // Set up dialog handler to reject the confirmation
    await browser.execute(() => {
      // Override confirm to return false (cancel)
      window.confirm = () => false;
    });

    // Click the delete button
    const deleteBtn = await todoItem.$(".action-btn--delete");
    await deleteBtn.waitForDisplayed({ timeout: 5000 });
    await deleteBtn.click();

    // Wait a moment
    await browser.pause(500);

    // Verify the todo still exists
    const existingTodo = await $(`article.todo-item*=${todoTitle}`);
    expect(await existingTodo.isExisting()).toBe(true);

    console.log(`Successfully canceled deletion of "${todoTitle}"`);
  });

  it("should delete a completed todo", async () => {
    // Create a todo
    const todoTitle = `Delete Completed Test ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Mark it as complete first
    const todoItem = await findTodoByTitle(todoTitle);
    const checkbox = await todoItem.$(".checkbox");
    await checkbox.click();

    // Wait for it to move to completed section
    await browser.pause(500);

    // Find it in the completed section
    const completedSection = await $(".section--completed");
    await completedSection.waitForDisplayed({ timeout: 5000 });

    const completedTodo = await completedSection.$(
      `article.todo-item*=${todoTitle}`
    );
    await completedTodo.waitForDisplayed({ timeout: 5000 });

    // Set up dialog handler to accept
    await browser.execute(() => {
      window.confirm = () => true;
    });

    // Delete the completed todo
    const deleteBtn = await completedTodo.$(".action-btn--delete");
    await deleteBtn.click();

    // Wait for deletion
    await browser.pause(500);

    // Verify the todo no longer exists anywhere
    const deletedTodo = await $(`article.todo-item*=${todoTitle}`);
    expect(await deletedTodo.isExisting()).toBe(false);

    console.log(`Successfully deleted completed todo "${todoTitle}"`);
  });

  it("should delete multiple todos independently", async () => {
    // Create multiple todos
    const todo1Title = `Multi Delete 1 ${Date.now()}`;
    const todo2Title = `Multi Delete 2 ${Date.now()}`;
    const todo3Title = `Multi Delete 3 ${Date.now()}`;

    await createTodo({ title: todo1Title });
    await createTodo({ title: todo2Title });
    await createTodo({ title: todo3Title });

    // Verify all three exist
    expect(await (await $(`article.todo-item*=${todo1Title}`)).isExisting()).toBe(true);
    expect(await (await $(`article.todo-item*=${todo2Title}`)).isExisting()).toBe(true);
    expect(await (await $(`article.todo-item*=${todo3Title}`)).isExisting()).toBe(true);

    // Set up dialog handler to accept
    await browser.execute(() => {
      window.confirm = () => true;
    });

    // Delete the second todo
    const todo2Item = await findTodoByTitle(todo2Title);
    const deleteBtn2 = await todo2Item.$(".action-btn--delete");
    await deleteBtn2.click();

    await browser.pause(500);

    // Verify todo2 is gone but todo1 and todo3 still exist
    expect(await (await $(`article.todo-item*=${todo1Title}`)).isExisting()).toBe(true);
    expect(await (await $(`article.todo-item*=${todo2Title}`)).isExisting()).toBe(false);
    expect(await (await $(`article.todo-item*=${todo3Title}`)).isExisting()).toBe(true);

    console.log(`Successfully deleted "${todo2Title}" while keeping others`);
  });
});
