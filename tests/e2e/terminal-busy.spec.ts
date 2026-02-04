import { expect } from "@wdio/globals";
import {
  createTempDbPath,
  cleanupTempDb,
  setupTestEnvironment,
  createTodo,
  findTodoByTitle,
} from "./helpers/test-setup";

describe("Terminal Busy Indicator", () => {
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

  it("should show busy indicator when terminal is executing a command", async () => {
    const todoTitle = `Terminal Busy Test ${Date.now()}`;

    // Create a todo
    await createTodo({ title: todoTitle });

    // Find the todo item
    const todoItem = await findTodoByTitle(todoTitle);
    await todoItem.scrollIntoView();
    await todoItem.moveTo();
    await browser.pause(300);

    // Click the terminal button to open terminal panel
    // Use JavaScript click because the button has opacity:0 by default and only shows on CSS hover
    const terminalBtn = await todoItem.$(".action-btn--terminal");
    await terminalBtn.waitForExist({ timeout: 5000 });
    await browser.execute((btn: Element) => (btn as HTMLElement).click(), terminalBtn);

    // Wait for terminal panel to appear
    const terminalPanel = await $(".terminal-panel.visible");
    await terminalPanel.waitForDisplayed({ timeout: 5000 });

    // Wait for terminal to initialize
    await browser.pause(1000);

    // Get the todo ID from the todo item's data attribute
    const todoId = await todoItem.getAttribute("data-todo-id");
    console.log(`Got todo ID: ${todoId}`);

    // Use Tauri's write_to_terminal command to send characters directly
    // This bypasses browser keyboard which doesn't work with xterm.js
    await browser.executeAsync((id: string, done: () => void) => {
      // @ts-expect-error - Tauri exposes invoke on window.__TAURI__
      window.__TAURI__.core.invoke("write_to_terminal", { todoId: id, data: "sleep 15\n" })
        .then(() => done())
        .catch(() => done());
    }, todoId);

    // Wait a moment for the command to start
    await browser.pause(1000);

    // Check that the busy indicator appears on the todo (polling happens every 5 seconds,
    // but let's wait for it)
    // We need to close the terminal panel first to see the todo item clearly
    // Actually, the indicator should be visible even with the terminal open

    // Wait for the polling to detect the busy state (up to 6 seconds to account for poll interval)
    let busyIndicator;
    let busyFound = false;
    for (let i = 0; i < 12; i++) {
      await browser.pause(500);
      busyIndicator = await todoItem.$(".terminal-busy-indicator");
      if (await busyIndicator.isExisting()) {
        busyFound = true;
        console.log(`Busy indicator found after ${(i + 1) * 500}ms`);
        break;
      }
    }

    expect(busyFound).toBe(true);

    // Verify the busy dot is visible
    const busyDot = await busyIndicator.$(".busy-dot");
    expect(await busyDot.isDisplayed()).toBe(true);

    console.log("Busy indicator is displayed while command is running");

    // Now wait for the command to complete (sleep 15) plus polling interval
    // Total wait: remaining sleep time + poll interval
    await browser.pause(16000);

    // After command completes, the indicator should change to idle
    // Wait for polling to detect idle state
    let idleIndicator;
    let idleFound = false;
    for (let i = 0; i < 12; i++) {
      await browser.pause(500);
      idleIndicator = await todoItem.$(".terminal-idle-indicator");
      if (await idleIndicator.isExisting()) {
        idleFound = true;
        console.log(`Idle indicator found after ${(i + 1) * 500}ms`);
        break;
      }
    }

    expect(idleFound).toBe(true);

    // Verify the idle dot is visible
    const idleDot = await idleIndicator.$(".idle-dot");
    expect(await idleDot.isDisplayed()).toBe(true);

    console.log("Idle indicator is displayed after command completes");
    console.log(`Successfully tested terminal busy indicator for "${todoTitle}"`);
  });

  it("should not show any indicator when no terminal session exists", async () => {
    const todoTitle = `No Terminal Test ${Date.now()}`;

    // Create a todo without opening a terminal
    await createTodo({ title: todoTitle });

    // Find the todo item
    const todoItem = await findTodoByTitle(todoTitle);

    // Wait a bit for any potential polling
    await browser.pause(1000);

    // Verify no busy or idle indicator exists
    const busyIndicator = await todoItem.$(".terminal-busy-indicator");
    const idleIndicator = await todoItem.$(".terminal-idle-indicator");

    expect(await busyIndicator.isExisting()).toBe(false);
    expect(await idleIndicator.isExisting()).toBe(false);

    console.log(`Verified no indicator shown for "${todoTitle}" without terminal session`);
  });

  it("should show idle indicator when terminal is at prompt", async () => {
    const todoTitle = `Terminal Idle Test ${Date.now()}`;

    // Create a todo
    await createTodo({ title: todoTitle });

    // Find the todo item
    const todoItem = await findTodoByTitle(todoTitle);
    await todoItem.scrollIntoView();
    await todoItem.moveTo();
    await browser.pause(300);

    // Click the terminal button to open terminal panel
    // Use JavaScript click because the button has opacity:0 by default and only shows on CSS hover
    const terminalBtn = await todoItem.$(".action-btn--terminal");
    await terminalBtn.waitForExist({ timeout: 5000 });
    await browser.execute((btn: Element) => (btn as HTMLElement).click(), terminalBtn);

    // Wait for terminal panel to appear
    const terminalPanel = await $(".terminal-panel.visible");
    await terminalPanel.waitForDisplayed({ timeout: 5000 });

    // Wait for terminal to initialize and shell to be ready
    await browser.pause(2000);

    // Don't run any command - terminal should be idle at prompt
    // Wait for polling to detect idle state (up to 6 seconds)
    let idleIndicator;
    let idleFound = false;
    for (let i = 0; i < 12; i++) {
      await browser.pause(500);
      idleIndicator = await todoItem.$(".terminal-idle-indicator");
      if (await idleIndicator.isExisting()) {
        idleFound = true;
        console.log(`Idle indicator found after ${(i + 1) * 500}ms`);
        break;
      }
    }

    expect(idleFound).toBe(true);

    // Verify the idle dot is visible
    const idleDot = await idleIndicator.$(".idle-dot");
    expect(await idleDot.isDisplayed()).toBe(true);

    console.log("Idle indicator is displayed when terminal is at prompt");
    console.log(`Successfully tested terminal idle indicator for "${todoTitle}"`);
  });
});
