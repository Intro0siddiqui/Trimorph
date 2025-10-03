import * as vscode from 'vscode';

// This method is called when your extension is activated
export function activate(context: vscode.ExtensionContext) {

    // Log a message to the console for verification
    console.log('Congratulations, your extension "trimorph-manager" is now active!');

    // The command has been defined in the package.json file
    // Now provide the implementation of the command with registerCommand
    // The commandId parameter must match the command field in package.json
    let disposable = vscode.commands.registerCommand('trimorph.switchJail', () => {
        // The code you place here will be executed every time your command is executed
        vscode.window.showInformationMessage('Switching Trimorph Jail!');
    });

    context.subscriptions.push(disposable);
}

// This method is called when your extension is deactivated
export function deactivate() {}