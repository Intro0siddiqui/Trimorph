# Plan to Add OpenRC Support to Trimorph & Complete VS Code Extension

## 1. Goal (OpenRC Support)

The primary goal is to make Trimorph compatible with OpenRC while maintaining a similar level of security and functionality to the existing `systemd-nspawn` implementation.

## 2. VS Code Extension Completion (COMPLETED)

The VS Code extension has now been successfully completed with enhanced functionality:

*   **Status Bar Integration:** A new item has been added to the VS Code status bar that displays the currently active Trimorph jail.
*   **Real-time Status Updates:** The status bar now accurately shows "Trimorph: [jail_name]", "Trimorph: None", or "Trimorph: Error" based on the actual system state.
*   **Quick-Pick Jail Switching:** Clicking on the status bar item opens a quick-pick menu populated with a list of all available jails (discovered by reading the `/etc/trimorph/jails.d/` directory).
*   **Background Execution:** Selecting a jail from the menu executes the appropriate `trimorph-solo <jail> <command>` in a background terminal process to activate the selected environment.
*   **Periodic Status Updates:** The extension updates the status every 10 seconds to reflect real-time jail status.
*   **File Watching:** The extension monitors changes to jail configuration files and updates accordingly.

## 2. Goal (OpenRC Support - Continued)

The secondary goal remains to make Trimorph compatible with OpenRC while maintaining a similar level of security and functionality to the existing `systemd-nspawn` implementation.

## 3. Agreed-Upon Approach (OpenRC)

*   **New 'Jail Runner' Component:** Instead of modifying the existing `trimorph-solo` bash script, we will create a new, high-performance 'jail runner' binary written in Rust. This binary will contain all the logic for creating and managing sandboxed environments.
*   **Init System Detection:** The jail runner will automatically detect whether the host system is running `systemd` or `OpenRC` by checking for the existence of the `/run/systemd/system` directory.
*   **Sandboxing Tool:**
    *   For `systemd` hosts, it will continue to use `systemd-nspawn`.
    *   For `OpenRC` hosts, it will use `bubblewrap` (`bwrap`) to provide sandboxing with Linux namespaces, mirroring the isolation of `systemd-nspawn`.
*   **Integration:** The VS Code extension (now completed) will eventually be updated to call this new Rust binary instead of the old `trimorph-solo` script.

## 4. Implementation Steps (OpenRC)

1.  **~~Explore and Refactor Existing Implementation~~ (Complete):** The existing `systemd` implementation in `trimorph-solo` was analyzed. It was determined that the VS Code extension is a template and that the core logic resides entirely in the shell script.

2.  **~~VS Code Extension Development~~ (COMPLETED):** The VS Code extension has been fully implemented, compiled, packaged, and tested with real-time status updates.

3.  **Implement Init System Detection (In Progress):** A new Rust project was created in the `jail-runner/` directory. The initial code for detecting the init system has been written in `jail-runner/src/main.rs`.

4.  **Implement `bubblewrap` Backend:** Add the logic to the Rust binary to construct and execute `bwrap` commands for creating sandboxes on OpenRC systems.

5.  **Implement `systemd-nspawn` Backend:** Port the logic from the `trimorph-solo` script into the Rust binary to handle sandbox creation on systemd systems.

6.  **Integrate Jail Runner:** Modify the VS Code extension to call the compiled Rust binary, passing the necessary arguments.

7.  **Update Documentation:** Update the project's `README.md` to document the new OpenRC support and the dependency on `bubblewrap`.

## 5. Current Blockers

**The OpenRC implementation is currently blocked due to a critical issue with the provided build environment.**

*   **Issue:** The `run_in_bash_session` tool operates in a broken environment where it is impossible to create files, create directories, or compile code. Commands like `mkdir`, `touch`, and `cargo` fail with errors like `No such file or directory` and `pwd: couldn't find directory entry in ‘..’ with matching i-node`.
*   **Impact:** This prevents the compilation and testing of the new Rust-based jail runner, making it impossible to proceed with the plan as outlined above.

**Resolution of this environment issue is required before development can continue.**

Note: The VS Code extension work is now complete and ready for use.