import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { exec } from 'child_process';

// This method is called when your extension is activated
export function activate(context: vscode.ExtensionContext) {
    console.log('Trimorph Manager extension is now active!');

    // Create a status bar item to show the active jail
    const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'trimorph.switchJail';
    statusBarItem.text = '$(server) Trimorph: Unknown';
    statusBarItem.tooltip = 'Click to switch Trimorph jail';
    statusBarItem.show();

    // Update the status bar with the current jail
    updateStatusBar(statusBarItem);

    // Set up periodic status updates
    const statusUpdateInterval = setInterval(() => {
        updateStatusBar(statusBarItem);
    }, 10000); // Update every 10 seconds

    // Register the command to switch jails
    const disposable = vscode.commands.registerCommand('trimorph.switchJail', async () => {
        try {
            // Read the contents of the /etc/trimorph/jails.d/ directory to get available jails
            const jailsDir = '/etc/trimorph/jails.d/';
            
            if (!fs.existsSync(jailsDir)) {
                vscode.window.showErrorMessage(`Jails directory does not exist: ${jailsDir}`);
                return;
            }

            const files = fs.readdirSync(jailsDir);
            const jails = files
                .filter(file => file.endsWith('.conf'))
                .map(file => path.parse(file).name);

            if (jails.length === 0) {
                vscode.window.showInformationMessage('No Trimorph jails found in /etc/trimorph/jails.d/');
                return;
            }

            // Use vscode.window.showQuickPick to display the list to the user
            const selectedJail = await vscode.window.showQuickPick(jails, {
                placeHolder: 'Select a Trimorph jail to activate...',
                canPickMany: false
            });

            if (selectedJail) {
                // Execute trimorph-solo command in the background
                const command = `trimorph-solo ${selectedJail} echo "Switched to ${selectedJail}"`;
                
                exec(command, (error, stdout, stderr) => {
                    if (error) {
                        vscode.window.showErrorMessage(`Error switching to jail ${selectedJail}: ${error.message}`);
                        return;
                    }
                    if (stderr) {
                        console.error(`stderr: ${stderr}`);
                        vscode.window.showWarningMessage(`Warning: ${stderr}`);
                    }
                    console.log(`stdout: ${stdout}`);
                    vscode.window.showInformationMessage(`Switched to Trimorph jail: ${selectedJail}`);
                    
                    // Update status bar after switching
                    updateStatusBar(statusBarItem);
                });
            }
        } catch (error) {
            vscode.window.showErrorMessage(`Error reading jails directory: ${(error as Error).message}`);
        }
    });

    context.subscriptions.push(disposable);
    
    // Set up a file watcher to update the status when jails change
    const watcher = vscode.workspace.createFileSystemWatcher('/etc/trimorph/jails.d/**/*.conf');
    watcher.onDidChange(() => updateStatusBar(statusBarItem));
    watcher.onDidCreate(() => updateStatusBar(statusBarItem));
    watcher.onDidDelete(() => updateStatusBar(statusBarItem));
    context.subscriptions.push(watcher);
    
    // Store interval ID to clear it on deactivation
    context.subscriptions.push({ dispose: () => clearInterval(statusUpdateInterval) });
}

// Update the status bar item with the current jail information
async function updateStatusBar(statusBarItem: vscode.StatusBarItem) {
    try {
        // Check which jails are currently active by looking for active systemd scopes
        const { exec } = require('child_process');
        const activeJails = await new Promise<string[]>((resolve, reject) => {
            exec('systemctl list-units --type=scope --state=active | grep trimorph-', (error: any, stdout: string, stderr: string) => {
                if (error) {
                    console.error('Error checking active jails:', stderr);
                    resolve([]);
                    return;
                }
                
                // Extract jail names from the systemctl output
                const lines = stdout.trim().split('\n').filter((line: string) => line.includes('trimorph-'));
                const jails = lines.map((line: string) => {
                    const jail = line.split('trimorph-')[1];
                    return jail.split('.scope')[0];
                }).filter((jail: string) => jail.length > 0);
                
                resolve(jails);
            });
        });

        if (activeJails.length > 0) {
            statusBarItem.text = `$(server) Trimorph: ${activeJails[0]}`;
            statusBarItem.tooltip = `Active Trimorph jail: ${activeJails[0]} - Click to switch`;
        } else {
            statusBarItem.text = '$(server) Trimorph: None';
            statusBarItem.tooltip = 'No active Trimorph jail - Click to switch';
        }
    } catch (error) {
        console.error('Error updating status bar:', error);
        statusBarItem.text = '$(server) Trimorph: Error';
        statusBarItem.tooltip = 'Error checking Trimorph status - Click to switch';
    }
}

// This method is called when your extension is deactivated
export function deactivate() {}