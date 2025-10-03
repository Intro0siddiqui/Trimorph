# Future Development Plan: Trimorph VS Code Extension

This document outlines the plan for creating a VS Code extension to manage Trimorph jails. This work was planned but blocked due to environmental issues with `npm`.

## 1. Goal

The primary goal is to provide a seamless, integrated way for developers to manage and switch between Trimorph jails directly from the VS Code editor, improving workflow efficiency.

## 2. Core Features

The extension will provide the following core features:

-   **Status Bar Integration:** A new item will be added to the VS Code status bar that displays the currently active Trimorph jail.
-   **Quick-Pick Jail Switching:** Clicking on the status bar item will open a quick-pick menu populated with a list of all available jails (discovered by reading the `/etc/trimorph/jails.d/` directory).
-   **Background Execution:** Selecting a jail from the menu will execute the appropriate `trimorph-solo <jail> <command>` in a background terminal process to activate the selected environment.

## 3. Technical Implementation Plan

The extension will be a separate software project housed in the `vscode-extension/` directory.

### 3.1 Project Setup (Completed)
-   **Directory:** `vscode-extension/`
-   **`package.json`:** Defines the extension's metadata, dependencies, and contributions (commands, UI elements).
-   **`tsconfig.json`:** Configures the TypeScript compiler.
-   **`.gitignore`:** Ignores `node_modules` and `out` directories.

### 3.2 Blocked Step: Dependency Installation
-   The `npm install` command consistently fails with a low-level `uv_cwd` error in the current development environment. This step must be resolved before compilation can proceed.

### 3.3 Implementation Steps (Post-Blocker)

1.  **Install Dependencies:** Run `npm install` to fetch the required `@types/vscode` and `typescript` packages.
2.  **Implement `extension.ts`:**
    -   Use `vscode.window.createStatusBarItem` to create the status bar UI element.
    -   Implement the `trimorph.switchJail` command.
    -   The command logic will:
        -   Read the contents of the `/etc/trimorph/jails.d/` directory to get a list of available jails.
        -   Use `vscode.window.showQuickPick` to display the list to the user.
        -   On selection, use `child_process.exec` to run the `trimorph-solo` command in the background.
    -   The `activate` function will register the command and initialize the status bar item.
3.  **Compile the Extension:** Run `npm run compile` to build the TypeScript source code into JavaScript in the `out/` directory.
4.  **Testing and Packaging:**
    -   The extension can be tested by opening the project in VS Code and running the "Run Extension" debug configuration.
    -   Once complete, it can be packaged into a `.vsix` file for distribution.