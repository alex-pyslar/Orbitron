import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { execFile, spawn } from 'child_process';

// ── Helpers ──────────────────────────────────────────────────────────────────

function getConfig() {
    return vscode.workspace.getConfiguration('orbitron');
}

function resolveExecutable(): string {
    return getConfig().get<string>('executablePath') ?? 'orbitron';
}

function useWsl(): boolean {
    return getConfig().get<boolean>('useWsl') ?? false;
}

function wslMountRoot(): string {
    return getConfig().get<string>('wslMountRoot') ?? '/mnt/c';
}

/** Convert a Windows absolute path to a WSL path, e.g. C:\foo → /mnt/c/foo */
function toWslPath(winPath: string): string {
    return winPath
        .replace(/^([A-Za-z]):/, (_, d) => `${wslMountRoot()}/${d.toLowerCase()}`)
        .replace(/\\/g, '/');
}

/**
 * Build the argv to run orbitron, optionally wrapping with wsl.
 * Returns [program, args[]] ready for spawn / execFile.
 */
function buildCommand(orbitronArgs: string[]): [string, string[]] {
    const exe = resolveExecutable();
    if (useWsl()) {
        return ['wsl', ['-e', 'bash', '-c', `${exe} ${orbitronArgs.join(' ')}`]];
    }
    return [exe, orbitronArgs];
}

// ── Output channel ───────────────────────────────────────────────────────────

let outputChannel: vscode.OutputChannel;

function getOutput(): vscode.OutputChannel {
    if (!outputChannel) {
        outputChannel = vscode.window.createOutputChannel('Orbitron');
    }
    return outputChannel;
}

function runInTerminal(cmd: string, cwd: string) {
    const terminal = vscode.window.createTerminal({
        name: 'Orbitron',
        cwd,
    });
    terminal.show();
    terminal.sendText(cmd);
}

// ── Commands ──────────────────────────────────────────────────────────────────

async function cmdBuild() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) { return; }

    const filePath = editor.document.fileName;
    if (!filePath.endsWith('.ot')) {
        vscode.window.showWarningMessage('Active file is not an Orbitron source file (.ot).');
        return;
    }

    const workspaceFolder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
    const cwd = workspaceFolder?.uri.fsPath ?? path.dirname(filePath);
    const hasManifest = fs.existsSync(path.join(cwd, 'orbitron.toml'));

    const backend = getConfig().get<string>('defaultBackend') ?? 'llvm';
    const verbose = getConfig().get<boolean>('verboseOutput') ? ['-v'] : [];
    const exe = resolveExecutable();

    let cmd: string;
    if (hasManifest) {
        cmd = `${exe} build --backend ${backend} ${verbose.join(' ')}`.trim();
    } else {
        const src = useWsl() ? toWslPath(filePath) : filePath;
        cmd = `${exe} ${src} --backend ${backend} ${verbose.join(' ')}`.trim();
    }

    getOutput().clear();
    getOutput().show(true);
    getOutput().appendLine(`> ${cmd}\n`);

    runInTerminal(cmd, cwd);
}

async function cmdRun() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) { return; }

    const filePath = editor.document.fileName;
    if (!filePath.endsWith('.ot')) {
        vscode.window.showWarningMessage('Active file is not an Orbitron source file (.ot).');
        return;
    }

    const workspaceFolder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
    const cwd = workspaceFolder?.uri.fsPath ?? path.dirname(filePath);
    const hasManifest = fs.existsSync(path.join(cwd, 'orbitron.toml'));

    const backend = getConfig().get<string>('defaultBackend') ?? 'llvm';
    const verbose = getConfig().get<boolean>('verboseOutput') ? ['-v'] : [];
    const exe = resolveExecutable();

    let cmd: string;
    if (hasManifest) {
        cmd = `${exe} run --backend ${backend} ${verbose.join(' ')}`.trim();
    } else {
        const src = useWsl() ? toWslPath(filePath) : filePath;
        const outName = path.basename(filePath, '.ot');
        const outPath = path.join(path.dirname(filePath), outName);
        const out = useWsl() ? toWslPath(outPath) : outPath;
        cmd = `${exe} ${src} -o ${out} --backend ${backend} ${verbose.join(' ')} && ${out}`.trim();
    }

    runInTerminal(cmd, cwd);
}

async function cmdFmt() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) { return; }

    await editor.document.save();

    const filePath = editor.document.fileName;
    if (!filePath.endsWith('.ot')) { return; }

    const src = useWsl() ? toWslPath(filePath) : filePath;
    const exe = resolveExecutable();
    const cwd = path.dirname(filePath);

    let program: string;
    let args: string[];

    if (useWsl()) {
        program = 'wsl';
        args = ['-e', 'bash', '-c', `${exe} fmt ${src}`];
    } else {
        program = exe;
        args = ['fmt', src];
    }

    execFile(program, args, { cwd }, (err, stdout, stderr) => {
        if (err) {
            vscode.window.showErrorMessage(`orbitron fmt failed: ${stderr || err.message}`);
            return;
        }
        vscode.window.showInformationMessage('File formatted.');
    });
}

async function cmdNewProject() {
    const name = await vscode.window.showInputBox({
        prompt: 'Project name',
        placeHolder: 'myapp',
        validateInput: v => /^[a-zA-Z_][a-zA-Z0-9_-]*$/.test(v) ? null : 'Use letters, digits, _ or -',
    });
    if (!name) { return; }

    const folderResult = await vscode.window.showOpenDialog({
        canSelectFolders: true,
        canSelectFiles: false,
        canSelectMany: false,
        openLabel: 'Select parent folder',
    });
    if (!folderResult || folderResult.length === 0) { return; }

    const parentDir = folderResult[0].fsPath;
    const exe = resolveExecutable();
    const cwd = parentDir;

    let program: string;
    let args: string[];

    if (useWsl()) {
        const wslParent = toWslPath(parentDir);
        program = 'wsl';
        args = ['-e', 'bash', '-c', `cd ${wslParent} && ${exe} new ${name}`];
    } else {
        program = exe;
        args = ['new', name];
    }

    execFile(program, args, { cwd }, (err, _stdout, stderr) => {
        if (err) {
            vscode.window.showErrorMessage(`orbitron new failed: ${stderr || err.message}`);
            return;
        }
        const projectDir = vscode.Uri.file(path.join(parentDir, name));
        vscode.commands.executeCommand('vscode.openFolder', projectDir, { forceNewWindow: false });
    });
}

// ── Format-on-save provider ──────────────────────────────────────────────────

class OrbitronFormatProvider implements vscode.DocumentFormattingEditProvider {
    provideDocumentFormattingEdits(
        document: vscode.TextDocument,
    ): Promise<vscode.TextEdit[]> {
        return new Promise((resolve, reject) => {
            const filePath = document.fileName;
            const src = useWsl() ? toWslPath(filePath) : filePath;
            const exe = resolveExecutable();
            const cwd = path.dirname(filePath);

            let program: string;
            let args: string[];

            if (useWsl()) {
                program = 'wsl';
                args = ['-e', 'bash', '-c', `${exe} fmt ${src} && cat ${src}`];
            } else {
                program = exe;
                args = ['fmt', src];
            }

            execFile(program, args, { cwd }, (err, stdout, stderr) => {
                if (err) {
                    reject(new Error(`orbitron fmt: ${stderr || err.message}`));
                    return;
                }
                // Re-read formatted file
                try {
                    const newText = fs.readFileSync(filePath, 'utf8');
                    const fullRange = document.validateRange(
                        new vscode.Range(0, 0, document.lineCount, 0),
                    );
                    resolve([vscode.TextEdit.replace(fullRange, newText)]);
                } catch {
                    resolve([]);
                }
            });
        });
    }
}

// ── Task provider ────────────────────────────────────────────────────────────

class OrbitronTaskProvider implements vscode.TaskProvider {
    static taskType = 'orbitron';

    provideTasks(): vscode.Task[] {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) { return []; }

        const cwd = workspaceFolders[0].uri.fsPath;
        const exe = resolveExecutable();
        const backend = getConfig().get<string>('defaultBackend') ?? 'llvm';

        const makeTask = (task: string, cmd: string): vscode.Task => {
            const def: vscode.TaskDefinition = { type: OrbitronTaskProvider.taskType, task };
            const shellExec = new vscode.ShellExecution(cmd, { cwd });
            const t = new vscode.Task(
                def,
                vscode.TaskScope.Workspace,
                task,
                'orbitron',
                shellExec,
                '$orbitron',
            );
            t.group = task === 'build' ? vscode.TaskGroup.Build : undefined;
            return t;
        };

        return [
            makeTask('build', `${exe} build --backend ${backend}`),
            makeTask('run',   `${exe} run   --backend ${backend}`),
            makeTask('fmt',   `${exe} fmt`),
        ];
    }

    resolveTask(task: vscode.Task): vscode.Task | undefined {
        const definition = task.definition as { type: string; task: string; backend?: string };
        if (definition.type === OrbitronTaskProvider.taskType) {
            return task;
        }
        return undefined;
    }
}

// ── Diagnostics (simple parse-error matcher) ─────────────────────────────────

let diagnosticCollection: vscode.DiagnosticCollection;

function runDiagnostics(document: vscode.TextDocument) {
    if (!document.fileName.endsWith('.ot')) { return; }

    const filePath = document.fileName;
    const src = useWsl() ? toWslPath(filePath) : filePath;
    const exe = resolveExecutable();
    const cwd = path.dirname(filePath);

    let program: string;
    let args: string[];

    if (useWsl()) {
        program = 'wsl';
        args = ['-e', 'bash', '-c', `${exe} ${src} --emit-llvm -o /dev/null 2>&1 || true`];
    } else {
        program = exe;
        args = [src, '--emit-llvm', '-o', '/dev/null'];
    }

    const proc = spawn(program, args, { cwd });
    let output = '';

    proc.stdout.on('data', (d: Buffer) => { output += d.toString(); });
    proc.stderr.on('data', (d: Buffer) => { output += d.toString(); });

    proc.on('close', () => {
        const diagnostics: vscode.Diagnostic[] = [];
        // Pattern: filename:line:col: error: message  OR  error: message
        const re = /(?:^|\n)(?:.*?:(\d+):(\d+):\s*)?(error|warning):\s*(.+)/g;
        let m: RegExpExecArray | null;
        while ((m = re.exec(output)) !== null) {
            const line   = m[1] ? Math.max(0, parseInt(m[1]) - 1) : 0;
            const col    = m[2] ? Math.max(0, parseInt(m[2]) - 1) : 0;
            const sev    = m[3] === 'error'
                ? vscode.DiagnosticSeverity.Error
                : vscode.DiagnosticSeverity.Warning;
            const msg    = m[4].trim();
            const range  = new vscode.Range(line, col, line, col + 1);
            diagnostics.push(new vscode.Diagnostic(range, msg, sev));
        }
        diagnosticCollection.set(document.uri, diagnostics);
    });
}

// ── Hover provider ───────────────────────────────────────────────────────────

const KEYWORD_DOCS: Record<string, string> = {
    var:       '**var** — immutable variable binding\n```orbitron\nvar x = 42;\nvar pi: f64 = 3.14;\n```',
    'var mut': '**var mut** — mutable variable binding\n```orbitron\nvar mut count = 0;\n```',
    const:     '**const** — compile-time constant\n```orbitron\nconst MAX: i64 = 1000;\n```',
    fn:        '**fn** — function declaration\n```orbitron\nfn add(a: i64, b: i64) -> i64 { return a + b; }\nfn double(n: i64) => n * 2;  // expression body\n```',
    async:     '**async** — marks a function as asynchronous\n```orbitron\nasync fn fetch(n: i64) -> i64 { ... }\nvar result = await fetch(100);\n```',
    await:     '**await** — suspends until async expression resolves\n```orbitron\nvar result = await compute(100);\n```',
    go:        '**go** — spawns a goroutine (background task)\n```orbitron\ngo { println("parallel"); };\n```',
    chan:       '**chan** — creates a channel for goroutine communication\n```orbitron\nvar ch = chan();\ngo { ch <- 42; };\nvar v = <-ch;\n```',
    defer:     '**defer** — defers execution until end of scope (LIFO)\n```orbitron\ndefer println("cleanup");  // runs last\n```',
    unless:    '**unless** — executes block when condition is FALSE\n```orbitron\nunless (x == 0) { println(100 / x); }\n```',
    repeat:    '**repeat** — repeat a block N times\n```orbitron\nrepeat 5 { counter += 1; }\n```',
    match:     '**match** — pattern matching\n```orbitron\nmatch status {\n    Status.Ok    => { println("ok"); }\n    _            => { println("other"); }\n}\n```',
    struct:    '**struct** — value type with fields (Go/Rust style)\n```orbitron\nstruct Point { public var x: i64, public var y: i64 }\nimpl Point { ... }\n```',
    class:     '**class** — reference type with constructor (Java/Kotlin style)\n```orbitron\nclass BankAccount {\n    init(amount: i64) { self.balance = amount; }\n}\nvar acc = new BankAccount(500);\n```',
    trait:     '**trait** — interface / type class (Rust/Swift style)\n```orbitron\ntrait Drawable { fn draw(self); }\nimpl Drawable for Circle { ... }\n```',
    impl:      '**impl** — implement methods for a struct or trait for a type\n```orbitron\nimpl Point { public fn dist_sq(self) -> i64 => self.x*self.x + self.y*self.y; }\nimpl Drawable for Circle { fn draw(self) { ... } }\n```',
    enum:      '**enum** — integer-backed enumeration (Rust/Swift style)\n```orbitron\nenum Direction { North, South, East, West }\n```',
    import:    '**import** — import a module or stdlib\n```orbitron\nimport "std/math";\nimport "std/bits";\n```',
    extern:    '**extern** — declare an external C function\n```orbitron\nextern fn socket(domain: i64, type_: i64, protocol: i64) -> i64;\n```',
    println:   '**println** — print value followed by newline\n```orbitron\nprintln(42);\nprintln($"x={x}");\n```',
    public:    '**public** — visible everywhere',
    private:   '**private** — visible only within the class/struct',
    protected: '**protected** — visible to subclasses',
    internal:  '**internal** — visible within the module',
    static:    '**static** — belongs to the type, not an instance\n```orbitron\nstatic var count: i64;\npublic static fn new(x: i64) -> Point { ... }\nPoint::count;\n```',
};

class OrbitronHoverProvider implements vscode.HoverProvider {
    provideHover(
        document: vscode.TextDocument,
        position: vscode.Position,
    ): vscode.Hover | undefined {
        const range = document.getWordRangeAtPosition(position, /[a-zA-Z_][a-zA-Z0-9_]*/);
        if (!range) { return; }
        const word = document.getText(range);
        const doc = KEYWORD_DOCS[word];
        if (!doc) { return; }
        return new vscode.Hover(new vscode.MarkdownString(doc));
    }
}

// ── Status bar item ──────────────────────────────────────────────────────────

let statusBarItem: vscode.StatusBarItem;

function updateStatusBar(editor: vscode.TextEditor | undefined) {
    if (!editor || !editor.document.fileName.endsWith('.ot')) {
        statusBarItem.hide();
        return;
    }
    const backend = getConfig().get<string>('defaultBackend') ?? 'llvm';
    statusBarItem.text = `$(play) Orbitron [${backend}]`;
    statusBarItem.tooltip = 'Click to run current file';
    statusBarItem.command = 'orbitron.run';
    statusBarItem.show();
}

// ── Extension lifecycle ───────────────────────────────────────────────────────

export function activate(context: vscode.ExtensionContext) {
    // Commands
    context.subscriptions.push(
        vscode.commands.registerCommand('orbitron.build',      cmdBuild),
        vscode.commands.registerCommand('orbitron.run',        cmdRun),
        vscode.commands.registerCommand('orbitron.fmt',        cmdFmt),
        vscode.commands.registerCommand('orbitron.newProject', cmdNewProject),
    );

    // Language features
    context.subscriptions.push(
        vscode.languages.registerDocumentFormattingEditProvider(
            { language: 'orbitron' },
            new OrbitronFormatProvider(),
        ),
        vscode.languages.registerHoverProvider(
            { language: 'orbitron' },
            new OrbitronHoverProvider(),
        ),
    );

    // Diagnostics
    diagnosticCollection = vscode.languages.createDiagnosticCollection('orbitron');
    context.subscriptions.push(diagnosticCollection);

    // Task provider
    context.subscriptions.push(
        vscode.tasks.registerTaskProvider(
            OrbitronTaskProvider.taskType,
            new OrbitronTaskProvider(),
        ),
    );

    // Status bar
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    context.subscriptions.push(statusBarItem);
    updateStatusBar(vscode.window.activeTextEditor);

    // Events
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(updateStatusBar),
        vscode.workspace.onDidSaveTextDocument(doc => {
            if (getConfig().get<boolean>('formatOnSave')) {
                if (doc.languageId === 'orbitron') {
                    vscode.commands.executeCommand('editor.action.formatDocument');
                }
            }
            runDiagnostics(doc);
        }),
        vscode.workspace.onDidOpenTextDocument(runDiagnostics),
        vscode.workspace.onDidChangeTextDocument(e => {
            // Debounce: run diagnostics only on full document changes
            if (e.contentChanges.length > 0) {
                runDiagnostics(e.document);
            }
        }),
    );

    // Run diagnostics for already-open .ot files
    vscode.workspace.textDocuments.forEach(runDiagnostics);
}

export function deactivate() {
    diagnosticCollection?.dispose();
    outputChannel?.dispose();
    statusBarItem?.dispose();
}
