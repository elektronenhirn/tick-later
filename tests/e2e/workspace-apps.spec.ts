import { expect } from "@wdio/globals";
import {
  createTempDbPath,
  cleanupTempDb,
  setupTestEnvironment,
  createTodo,
  findTodoByTitle,
} from "./helpers/test-setup";

describe("Workspace Apps", () => {
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

  it("should create a todo with workspace apps configured", async () => {
    const todoTitle = `Workspace App Test ${Date.now()}`;

    await createTodo({
      title: todoTitle,
      workspaceApps: [
        { command: "echo test", workingDir: "/tmp" },
      ],
    });

    // Find the todo
    const todoItem = await findTodoByTitle(todoTitle);
    expect(await todoItem.isDisplayed()).toBe(true);

    // Verify the launch workspace button is displayed
    await todoItem.scrollIntoView();
    await todoItem.moveTo();
    await browser.pause(300);

    const launchBtn = await todoItem.$(".action-btn--workspace");
    expect(await launchBtn.isExisting()).toBe(true);

    console.log(`Successfully created todo "${todoTitle}" with workspace apps`);
  });

  it("should not show launch button when no workspace apps are configured", async () => {
    const todoTitle = `No Workspace Apps ${Date.now()}`;

    await createTodo({
      title: todoTitle,
    });

    // Find the todo
    const todoItem = await findTodoByTitle(todoTitle);

    // Hover to reveal action buttons
    await todoItem.scrollIntoView();
    await todoItem.moveTo();
    await browser.pause(300);

    // Verify the launch workspace button is NOT displayed
    const launchBtn = await todoItem.$(".action-btn--workspace");
    expect(await launchBtn.isExisting()).toBe(false);

    console.log(`Verified no launch button for "${todoTitle}" without workspace apps`);
  });

  it("should show correct app count in launch button tooltip", async () => {
    const todoTitle = `Multi App Test ${Date.now()}`;

    await createTodo({
      title: todoTitle,
      workspaceApps: [
        { command: "echo app1" },
        { command: "echo app2" },
        { command: "echo app3" },
      ],
    });

    // Find the todo
    const todoItem = await findTodoByTitle(todoTitle);

    // Hover to reveal action buttons
    await todoItem.scrollIntoView();
    await todoItem.moveTo();
    await browser.pause(300);

    const launchBtn = await todoItem.$(".action-btn--workspace");
    const title = await launchBtn.getAttribute("title");

    // Should show "3 apps" (plural)
    expect(title).toContain("3 apps");

    console.log(`Verified launch button tooltip shows correct app count for "${todoTitle}"`);
  });

  it("should persist workspace apps after editing todo", async () => {
    const todoTitle = `Persist Workspace Apps ${Date.now()}`;

    await createTodo({
      title: todoTitle,
      workspaceApps: [
        { command: "echo persisted", workingDir: "/tmp" },
      ],
    });

    // Find the todo and double-click to edit
    const todoItem = await findTodoByTitle(todoTitle);
    await todoItem.scrollIntoView();
    await todoItem.doubleClick();

    // Wait for edit modal
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Change the title
    const newTitle = `Updated ${todoTitle}`;
    const titleInput = await $("#title");
    await titleInput.clearValue();
    await titleInput.setValue(newTitle);

    // Submit the form
    const submitBtn = await $(".submit-btn");
    await submitBtn.scrollIntoView();
    await submitBtn.click();

    // Wait for modal to close
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Find the updated todo
    const updatedTodo = await findTodoByTitle(newTitle);

    // Hover to reveal action buttons
    await updatedTodo.scrollIntoView();
    await updatedTodo.moveTo();
    await browser.pause(300);

    // Verify the launch button is still there (workspace apps persisted)
    const launchBtn = await updatedTodo.$(".action-btn--workspace");
    expect(await launchBtn.isExisting()).toBe(true);

    console.log(`Verified workspace apps persisted after editing "${newTitle}"`);
  });

  it("should open workspace apps panel in edit mode with existing apps", async () => {
    const todoTitle = `Edit Workspace Apps ${Date.now()}`;

    await createTodo({
      title: todoTitle,
      workspaceApps: [
        { command: "echo existing", workingDir: "/home" },
      ],
    });

    // Find the todo and double-click to edit
    const todoItem = await findTodoByTitle(todoTitle);
    await todoItem.scrollIntoView();
    await todoItem.doubleClick();

    // Wait for edit modal
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // The workspace apps toggle should indicate apps are configured
    const workspaceToggle = await $(".workspace-toggle");
    await workspaceToggle.scrollIntoView();
    const toggleText = await workspaceToggle.getText();
    expect(toggleText).toContain("1 app(s) configured");

    // Check if panel is already visible (it might be auto-opened in edit mode)
    let workspacePanel = await $(".workspace-apps-panel");
    const isAlreadyVisible = await workspacePanel.isDisplayed();

    if (!isAlreadyVisible) {
      // Open the workspace apps panel
      await workspaceToggle.click();
      await browser.pause(500);
      await workspacePanel.waitForDisplayed({ timeout: 5000 });
    }

    const commandInput = await $(".app-input--command");
    const commandValue = await commandInput.getValue();
    expect(commandValue).toBe("echo existing");

    const dirInput = await $(".app-input--dir");
    const dirValue = await dirInput.getValue();
    expect(dirValue).toBe("/home");

    // Close the modal
    const closeBtn = await $(".close-btn");
    await closeBtn.click();
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    console.log(`Verified workspace apps panel shows existing apps for "${todoTitle}"`);
  });

  it("should remove workspace apps when all are deleted", async () => {
    const todoTitle = `Remove Workspace Apps ${Date.now()}`;

    await createTodo({
      title: todoTitle,
      workspaceApps: [
        { command: "echo to-be-removed" },
      ],
    });

    // Find the todo and double-click to edit
    const todoItem = await findTodoByTitle(todoTitle);
    await todoItem.scrollIntoView();
    await browser.pause(300);
    await todoItem.doubleClick();

    // Wait for edit modal
    const modal = await $(".modal-content");
    await modal.waitForDisplayed({ timeout: 5000 });

    // Open the workspace apps panel (check if already visible first)
    const workspaceToggle = await $(".workspace-toggle");
    await workspaceToggle.scrollIntoView();

    let workspacePanel = await $(".workspace-apps-panel");
    const isAlreadyVisible = await workspacePanel.isDisplayed();

    if (!isAlreadyVisible) {
      await workspaceToggle.click();
      await browser.pause(500);
      await workspacePanel.waitForDisplayed({ timeout: 5000 });
    }

    // Click the remove button on the app entry
    const removeBtn = await $(".app-remove-btn");
    await removeBtn.scrollIntoView();
    await removeBtn.click();
    await browser.pause(300);

    // Submit the form
    const submitBtn = await $(".submit-btn");
    await submitBtn.scrollIntoView();
    await browser.pause(200);
    await submitBtn.click();

    // Wait for modal to close
    await modal.waitForDisplayed({ reverse: true, timeout: 5000 });

    // Find the todo again
    const updatedTodo = await findTodoByTitle(todoTitle);

    // Hover to reveal action buttons
    await updatedTodo.scrollIntoView();
    await updatedTodo.moveTo();
    await browser.pause(300);

    // Verify the launch button is gone (no more workspace apps)
    const launchBtn = await updatedTodo.$(".action-btn--workspace");
    expect(await launchBtn.isExisting()).toBe(false);

    console.log(`Verified workspace apps removed for "${todoTitle}"`);
  });

  it("should create todo with workspace apps and virtual desktop", async () => {
    const todoTitle = `Workspace and Desktop ${Date.now()}`;

    // Test with virtualDesktop enabled
    await createTodo({
      title: todoTitle,
      virtualDesktop: 0, // Desktop 1 - re-enabled
      workspaceApps: [
        { command: "echo with-desktop" },
      ],
    });

    // Find the todo
    const todoItem = await findTodoByTitle(todoTitle);

    // Hover to reveal action buttons
    await todoItem.scrollIntoView();
    await todoItem.moveTo();
    await browser.pause(300);

    // Check if workspace button exists (this should work on all platforms)
    const launchBtn = await todoItem.$(".action-btn--workspace");
    expect(await launchBtn.isExisting()).toBe(true);

    // Check if desktop button exists (only on Linux)
    const desktopBtn = await todoItem.$(".action-btn--desktop");
    if (await desktopBtn.isExisting()) {
      console.log(`Verified todo "${todoTitle}" has both workspace apps and virtual desktop`);
    } else {
      console.log(`Verified todo "${todoTitle}" has workspace apps (virtual desktop not available on this platform)`);
    }
  });
});
