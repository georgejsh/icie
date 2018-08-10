'use strict';
import * as vscode from 'vscode';
import { homedir } from 'os';
import * as fs from 'fs';
import * as fs2 from 'fs-extra';
import * as ci from './ci';

// this method is called when your extension is activated
// your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
    console.log('Congratulations, your extension "icie" is now active!');

    let icie = new ICIE();

    // The command has been defined in the package.json file
    // Now provide the implementation of the command with  registerCommand
    // The commandId parameter must match the command field in package.json
    let disposable = vscode.commands.registerCommand('icie.build', () => {
        icie.triggerBuild();
    });
    let disposable2 = vscode.commands.registerCommand('icie.test', () => {
        icie.triggerTest();
    });
    let disposable3 = vscode.commands.registerCommand('icie.init', () => {
        icie.triggerInit();
    });
    let disposable4 = vscode.commands.registerCommand('icie.submit', () => {
        icie.triggerSubmit();
    });

    context.subscriptions.push(icie);
    context.subscriptions.push(disposable);
    context.subscriptions.push(disposable2);
    context.subscriptions.push(disposable3);
    context.subscriptions.push(disposable4);
    icie.launch();
}

// this method is called when your extension is deactivated
export function deactivate() {
}

class ICIE {

    ci: ci.Ci;

    public constructor() {
        this.ci = new ci.Ci;
    }

    public async launch(): Promise<void> {
        let _config: Promise<ICIEConfig> = ICIEConfig.load();
        let source = await then2promise(vscode.workspace.openTextDocument(this.getMainSource()));
            let editor = await then2promise(vscode.window.showTextDocument(source));
            let oldPosition = editor.selection.active;
            let config = await _config;
            let newPosition = oldPosition.with(config.template.start.row - 1, config.template.start.column - 1);
            let newSelection = new vscode.Selection(newPosition, newPosition);
            editor.selection = newSelection;
    }
    public async triggerBuild(): Promise<void> {
        console.log(`ICIE.@triggerBuild`);
        await this.assureAllSaved();
        let source = this.getMainSource();
        await this.ci.build(source);
    }
    public async triggerTest(): Promise<boolean> {
        await this.assureCompiled();
        let executable = this.getMainExecutable();
        let testdir = this.getTestDirectory();
        console.log(`[ICIE.triggerTest] Checking ${executable} agains ${testdir}`);
        return await this.ci.test(executable, testdir);
    }

    public async triggerInit(): Promise<void> {
        let task_url = await this.askDescriptionURL();
        let project_name = await this.randomProjectName(5);
        let project_dir = homedir() + '/' + project_name;
        await mkdir(project_dir);
        await this.ci.init(task_url, project_dir, domain => this.respondAuthreq(domain));
        let manifest = new ICIEManifest(task_url);
        await manifest.save(project_dir + '/.icie');
        let config = await ICIEConfig.load();
        await fs2.copy(config.template.path, project_dir + '/' + this.getPreferredMainSource());
        await vscode.commands.executeCommand('vscode.openFolder', vscode.Uri.file(project_dir), false);
    }
    public async triggerSubmit(): Promise<void> {
        console.log('ICIE Submit triggered');
        let tests_succeded = await this.triggerTest();
        if (tests_succeded) {
            let manifest = await ICIEManifest.load();
            console.log(`ICIE.@triggerSubmit.#manifest = ${manifest}`);
            await this.ci.submit(this.getMainSource(), manifest.task_url, authreq => this.respondAuthreq(authreq));
        }
    }
    private async respondAuthreq(authreq: ci.AuthRequest): Promise<ci.AuthResponse> {
        let username = await inputbox({ prompt: `Username at ${authreq.domain}` });
        let password = await inputbox({ prompt: `Password for ${username} at ${authreq.domain}`});
        return { username, password };
    }

    public dispose() {

    }

    private async requiresCompilation(): Promise<boolean> {
        console.log(`ICIE.@assureCompiled`);
        let src = this.getMainSource();
        console.log(`[ICIE.assureCompiled] Checking whether ${src} is compiled`);
        let exe = this.getMainExecutable();
        await this.assureAllSaved();
        console.log(`ICIE.@assureCompiled All files have been saved`);
        let statsrc = await file_stat(src);
        try {
            var statexe = await file_stat(exe);
        } catch (err) {
            console.log(`ICIE.@assureCompiled ${src} needs compiling`);
            return true;
        }
        if (statsrc.mtime <= statexe.mtime) {
            console.log(`[ICIE.assureCompiled] ${src} was compiled already`);
            return false;
        } else {
            console.log(`[ICIE.assureCompiled] ${src} needs compiling`);
            return true;
        }
    }

    private async assureCompiled(): Promise<void> {
        console.log(`ICIE.@assureCompiled`);
        if (await this.requiresCompilation()) {
            await this.triggerBuild();
        }
    }
    private async assureAllSaved(): Promise<void> {
        return vscode.workspace.saveAll(false).then(something => {});
    }

    private askDescriptionURL(): Promise<string> {
        let options: vscode.InputBoxOptions = {
            prompt: 'Task description URL: '
        };
        return new Promise((resolve, reject) => {
            vscode.window.showInputBox(options).then(value => {
                if (value) {
                    resolve(value);
                } else {
                    reject('problem description url not entered');
                }
            });
        });
    }

    private async randomProjectName(tries: number): Promise<string> {
        for (; tries>0; --tries) {
            let name = this.randomName();
            if (!await file_exists(homedir() + '/' + name)) {
                return name;
            }
        }
        return Promise.reject('failed to find free project name');
    }
    private randomName(): string {
        let adjectives = [
            "playful",
            "shining",
            "sparkling",
            "rainbow",
            "kawaii",
            "superb",
            "amazing",
            "glowing",
            "blessed",
            "smiling",
        ];
        let animals = [
            "capybara",
            "squirrel",
            "spider",
            "anteater",
            "hamster",
            "whale",
            "eagle",
            "zebra",
            "dolphin",
            "hedgehog",
        ];
        return choice(adjectives) + '-' + choice(animals);
    }

    private getMainSource(): string {
        let editor = vscode.window.activeTextEditor;
        if (!editor) {
            return vscode.workspace.rootPath + '/' + this.getPreferredMainSource();
        }
        let doc = editor.document;
        return doc.fileName.toString();
    }
    private getMainExecutable(): string {
        let source = this.getMainSource();
        return source.substr(0, source.length - 4) + '.e';
    }
    private getTestDirectory(): string {
        let path = vscode.workspace.rootPath;
        console.log('ICIE Build: getTestDirectory.path = ' + path);
        if (!path) {
            throw 'ICIE Build: path not found';
        }
        return path;
    }
    private getPreferredMainSource(): string {
        return "main.cpp";
    }

}

class ICIEManifest {

    public task_url: string;

    public constructor(task_url: string) {
        this.task_url = task_url;
    }

    static async load(): Promise<ICIEManifest> {
        let data = await readFile(vscode.workspace.rootPath + '/.icie', 'utf8');
        let json = JSON.parse(data.toString());
        return new ICIEManifest(json.task_url);
    }
    public save(path: string): Promise<void> {
        return new Promise((resolve, reject) => fs.writeFile(path, JSON.stringify(this), err1 => {
            if (err1) {
                reject(err1);
            } else {
                resolve();
            }
        }));
    }

}

class TextFilePos {
    public row: number;
    public column: number;
    public constructor(row: number, column: number) {
        this.row = row;
        this.column = column;
    }
}

class ICIEConfigTemplate {
    public path: string;
    public start: TextFilePos;
    public constructor(path: string, start: TextFilePos) {
        this.path = path;
        this.start = start;
    }
}

class ICIEConfig {

    public template: ICIEConfigTemplate;

    public constructor(template: ICIEConfigTemplate) {
        this.template = template;
    }

    static async load(): Promise<ICIEConfig> {
        let data = await readFile(ICIEConfig.prototype.getConfigPath(), 'utf8');
        let json = JSON.parse(data.toString());
        return new ICIEConfig(new ICIEConfigTemplate(json.template.path, new TextFilePos(json.template.start.row, json.template.start.column)));
    }
    public save(path: string): Promise<void> {
        return new Promise((resolve, reject) => fs.writeFile(path, JSON.stringify(this), err1 => {
            if (err1) {
                reject(err1);
            } else {
                resolve();
            }
        }));
    }

    private getConfigPath(): string {
        return homedir() + '/.config/icie/config.json';
    }

}

function choice<T>(xs: T[]): T {
    return xs[Math.floor(Math.random() * xs.length)];
}
function file_exists(path: string): Promise<boolean> {
    return new Promise(resolve => {
        fs.exists(path, resolve);
    });
}
function file_stat(path: string): Promise<fs.Stats> {
    console.log(`@file_stat`);
    return new Promise((resolve, reject) => {
        console.log(`@file_stat Promise started`);
        fs.stat(path, (err, stats) => {
            if (err) {
                console.log(`@file_stat.#err = ${err}`);
                reject(err);
            } else {
                console.log(`@file_stat.#stats = ${stats}`);
                resolve(stats);
            }
        })
    });
}
function mkdir(path: string): Promise<void> {
    return new Promise((resolve, reject) => {
        fs.mkdir(path, err => {
            if (err) {
                reject(err);
            } else {
                resolve();
            }
        });
    });
}
function readFile(filename: string, encoding: string): Promise<string> {
    return new Promise((resolve, reject) => {
        fs.readFile(filename, encoding, (err, data) => {
            if (err) {
                reject(err);
            } else {
                resolve(data);
            }
        })
    });
}
function then2promise<T>(t: Thenable<T>): Promise<T> {
    return new Promise((resolve, reject) => {
        t.then(val => resolve(val), reason => reject(reason));
    });
}
async function inputbox(options: vscode.InputBoxOptions): Promise<string> {
    let maybe = await then2promise(vscode.window.showInputBox(options));
    if (maybe !== undefined) {
        return maybe;
    } else {
        throw new Error("did not get input on input box");
    }
}