import { expect } from "@wdio/globals";
import {
  createTempDbPath,
  cleanupTempDb,
  setupTestEnvironment,
  createTodo,
} from "./helpers/test-setup";

describe("Search Feature", () => {
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

  it("should find a todo by title using the search dialog", async () => {
    // Create a todo with a unique title
    const todoTitle = `Searchable Todo ${Date.now()}`;
    await createTodo({ title: todoTitle });

    // Click the search button in the header
    const searchBtn = await $(".header-btn[title*='Search']");
    await searchBtn.waitForClickable({ timeout: 5000 });
    await searchBtn.click();

    // Wait for search modal to appear
    const searchModal = await $(".modal-content--search");
    await searchModal.waitForDisplayed({ timeout: 5000 });

    // Enter search query
    const searchInput = await $(".search-input");
    await searchInput.waitForDisplayed();
    await searchInput.setValue(todoTitle.substring(0, 15)); // Search with partial title

    // Click the Find button
    const findBtn = await $(".search-submit");
    await findBtn.click();

    // Wait for modal to close
    await searchModal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Verify the todo is highlighted
    const todoItem = await $(`article.todo-item*=${todoTitle}`);
    await todoItem.waitForDisplayed({ timeout: 5000 });

    const todoClasses = await todoItem.getAttribute("class");
    expect(todoClasses).toContain("todo-item--highlighted");

    console.log(`Successfully found and highlighted todo "${todoTitle}"`);
  });

  it("should find a todo by description using search", async () => {
    // Create a todo with a unique description
    const todoTitle = `Todo With Description ${Date.now()}`;
    const todoDescription = `UniqueDescription${Date.now()}`;
    await createTodo({ title: todoTitle, description: todoDescription });

    // Open search dialog with Ctrl+F
    await browser.keys(["Control", "f"]);

    // Wait for search modal to appear
    const searchModal = await $(".modal-content--search");
    await searchModal.waitForDisplayed({ timeout: 5000 });

    // Search by description content
    const searchInput = await $(".search-input");
    await searchInput.setValue(todoDescription.substring(0, 20));

    // Submit search
    const findBtn = await $(".search-submit");
    await findBtn.click();

    // Wait for modal to close
    await searchModal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Verify the todo is found and highlighted
    const todoItem = await $(`article.todo-item*=${todoTitle}`);
    await todoItem.waitForDisplayed({ timeout: 5000 });

    const todoClasses = await todoItem.getAttribute("class");
    expect(todoClasses).toContain("todo-item--highlighted");

    console.log(
      `Successfully found todo by description: "${todoDescription}"`
    );
  });

  it("should open search with Ctrl+F keyboard shortcut", async () => {
    // Press Ctrl+F
    await browser.keys(["Control", "f"]);

    // Verify search modal appears
    const searchModal = await $(".modal-content--search");
    await searchModal.waitForDisplayed({ timeout: 5000 });

    // Verify search input is focused
    const searchInput = await $(".search-input");
    const isFocused = await searchInput.isFocused();
    expect(isFocused).toBe(true);

    // Close with Escape
    await browser.keys(["Escape"]);
    await searchModal.waitForDisplayed({ reverse: true, timeout: 5000 });

    console.log("Successfully opened and closed search with keyboard");
  });
});
