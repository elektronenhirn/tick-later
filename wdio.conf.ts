import type { Options } from "@wdio/types";
import { spawn, spawnSync, ChildProcess } from "child_process";
import { platform } from "os";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

let tauriDriver: ChildProcess | null = null;
let devServer: ChildProcess | null = null;

// Determine the path to the Tauri application binary (debug build)
function getAppPath(): string {
  const targetDir = join(__dirname, "src-tauri", "target", "debug");

  switch (platform()) {
    case "linux":
      return join(targetDir, "tick-later");
    case "win32":
      return join(targetDir, "tick-later.exe");
    case "darwin":
      return join(
        targetDir,
        "bundle",
        "macos",
        "Tick Later.app",
        "Contents",
        "MacOS",
        "Tick Later"
      );
    default:
      throw new Error(`Unsupported platform: ${platform()}`);
  }
}

export const config: Options.Testrunner = {
  // Runner configuration
  runner: "local",
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: {
      transpileOnly: true,
      project: "./tsconfig.json",
    },
  },

  // Test files
  specs: ["./tests/e2e/**/*.spec.ts"],
  exclude: [],

  // Capabilities
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: getAppPath(),
      },
    },
  ],

  // Test framework
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },

  // Reporters
  reporters: ["spec"],

  // WebDriver connection
  hostname: "127.0.0.1",
  port: 4444,

  // Logging
  logLevel: "info",

  // Wait for application
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  // Hooks
  onPrepare: async function () {
    // Build the Rust backend and start the dev server
    const { execSync } = await import("child_process");

    try {
      console.log("Building Tauri Rust backend...");
      execSync("cargo build", {
        cwd: join(__dirname, "src-tauri"),
        stdio: "inherit",
      });
      console.log("Rust backend build completed.");

      // Start the Vite dev server
      console.log("Starting Vite dev server...");
      devServer = spawn("npm", ["run", "dev"], {
        cwd: __dirname,
        stdio: ["ignore", "pipe", "pipe"],
        shell: true,
      });

      devServer.stdout?.on("data", (data) => {
        console.log(`Vite: ${data}`);
      });

      devServer.stderr?.on("data", (data) => {
        console.error(`Vite stderr: ${data}`);
      });

      // Wait for dev server to be ready
      await new Promise((resolve) => setTimeout(resolve, 5000));
      console.log("Dev server started.");
    } catch (error) {
      console.error("Failed to build/start:", error);
      throw error;
    }
  },

  beforeSession: async function () {
    // Start tauri-driver before each test session
    console.log("Starting tauri-driver...");

    tauriDriver = spawn("tauri-driver", [], {
      stdio: ["ignore", "pipe", "pipe"],
    });

    tauriDriver.stdout?.on("data", (data) => {
      console.log(`tauri-driver stdout: ${data}`);
    });

    tauriDriver.stderr?.on("data", (data) => {
      console.error(`tauri-driver stderr: ${data}`);
    });

    // Wait for tauri-driver to be ready
    await new Promise((resolve) => setTimeout(resolve, 2000));
    console.log("tauri-driver started.");
  },

  afterSession: async function () {
    // Clean up tauri-driver after each test session
    if (tauriDriver) {
      console.log("Stopping tauri-driver...");
      tauriDriver.kill();
      tauriDriver = null;
    }
  },

  onComplete: async function () {
    // Clean up dev server after all tests
    if (devServer) {
      console.log("Stopping dev server...");
      devServer.kill();
      devServer = null;
    }
  },
};
