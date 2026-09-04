import fs from 'node:fs';
import path from 'node:path';
import chalk from 'chalk';
import type { Command } from 'commander';
import type { RtbConfig } from '../types/config.js';
import type { CliContext } from '../types/context.js';
import { ConfigMissingError } from '../errors.js';
import { outputJson } from '../utils/output.js';

export interface MaintenanceTaskContext {
  config: RtbConfig;
  configPath?: string;
  rootDir?: string;
  isReportOnly?: boolean;
  isFull?: boolean;
  isJson?: boolean;
}

export interface MaintenanceTaskResult {
  task: string;
  success: boolean;
  message: string;
  details?: unknown;
}

export type MaintenanceTaskHandler = (
  ctx: MaintenanceTaskContext
) => Promise<MaintenanceTaskResult> | MaintenanceTaskResult;

export class MaintenanceTaskRegistry {
  private tasks = new Map<string, MaintenanceTaskHandler>();

  constructor() {
    this.registerBuiltinTasks();
  }

  public registerTask(name: string, handler: MaintenanceTaskHandler): void {
    this.tasks.set(name.toLowerCase(), handler);
  }

  public hasTask(name: string): boolean {
    return this.tasks.has(name.toLowerCase());
  }

  public async runTask(name: string, ctx: MaintenanceTaskContext): Promise<MaintenanceTaskResult> {
    const handler = this.tasks.get(name.toLowerCase());
    if (!handler) {
      return {
        task: name,
        success: false,
        message: `Maintenance task '${name}' is not registered.`,
      };
    }
    return await handler(ctx);
  }

  public async runAll(ctx: MaintenanceTaskContext): Promise<MaintenanceTaskResult[]> {
    const results: MaintenanceTaskResult[] = [];
    for (const [name, handler] of this.tasks.entries()) {
      try {
        const res = await handler(ctx);
        results.push(res);
      } catch (err: unknown) {
        results.push({
          task: name,
          success: false,
          message: `Task failed: ${err instanceof Error ? err.message : String(err)}`,
        });
      }
    }
    return results;
  }

  private registerBuiltinTasks(): void {
    // 1. Guard Task
    this.registerTask('guard', (ctx) => {
      const targetRoot = ctx.rootDir || (process.platform === 'win32' ? 'D:\\' : process.env.HOME || '/');
      const unorganized: string[] = [];

      if (fs.existsSync(targetRoot)) {
        try {
          const entries = fs.readdirSync(targetRoot, { withFileTypes: true });
          for (const e of entries) {
            // Check top-level loose files or non-standard directories
            if (e.isFile() && !e.name.startsWith('.')) {
              unorganized.push(e.name);
            }
          }
        } catch {}
      }

      return {
        task: 'guard',
        success: true,
        message: unorganized.length === 0
          ? `Root drive guardrail clean: ${targetRoot}`
          : `Found ${unorganized.length} unorganized files in ${targetRoot}`,
        details: { targetRoot, unorganized },
      };
    });

    // 2. Backup Task (Config backup)
    this.registerTask('backup', (ctx) => {
      const backupRoot = ctx.config.backupRoot || path.join(process.cwd(), 'backups');
      const configBackupDir = path.join(backupRoot, 'configs');

      if (!fs.existsSync(configBackupDir)) {
        fs.mkdirSync(configBackupDir, { recursive: true });
      }

      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
      const targetFile = path.join(configBackupDir, `rtb.config-${timestamp}.json`);

      let backedUpPath = '';
      if (ctx.configPath && fs.existsSync(ctx.configPath)) {
        fs.copyFileSync(ctx.configPath, targetFile);
        backedUpPath = targetFile;
      } else {
        fs.writeFileSync(targetFile, JSON.stringify(ctx.config, null, 2), 'utf-8');
        backedUpPath = targetFile;
      }

      return {
        task: 'backup',
        success: true,
        message: `Config backed up to ${backedUpPath}`,
        details: { backupPath: backedUpPath },
      };
    });

    // 3. Env Task (Backup .env files)
    this.registerTask('env', (ctx) => {
      const backupRoot = ctx.config.backupRoot || path.join(process.cwd(), 'backups');
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
      const envBackupDir = path.join(backupRoot, 'env-backups', timestamp);

      if (!fs.existsSync(envBackupDir)) {
        fs.mkdirSync(envBackupDir, { recursive: true });
      }

      const backedUpFiles: string[] = [];

      const searchRoots: string[] = [];
      if (ctx.config.projectRoots) {
        for (const entry of Object.values(ctx.config.projectRoots)) {
          if (entry.path && fs.existsSync(entry.path)) {
            searchRoots.push(entry.path);
          }
        }
      }

      for (const root of searchRoots) {
        try {
          const projects = fs.readdirSync(root, { withFileTypes: true });
          for (const proj of projects) {
            if (!proj.isDirectory()) continue;
            const projDir = path.join(root, proj.name);
            const files = fs.readdirSync(projDir);
            for (const f of files) {
              if (f.startsWith('.env')) {
                const src = path.join(projDir, f);
                const destName = `${proj.name}-${f}`;
                const dest = path.join(envBackupDir, destName);
                fs.copyFileSync(src, dest);
                backedUpFiles.push(dest);
              }
            }
          }
        } catch {}
      }

      return {
        task: 'env',
        success: true,
        message: `Backed up ${backedUpFiles.length} .env files to ${envBackupDir}`,
        details: { envBackupDir, count: backedUpFiles.length, files: backedUpFiles },
      };
    });
  }
}

export function registerBackupCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('backup')
    .description('Full workspace configuration backup')
    .option('--json', 'Output backup results in JSON format')
    .action(async (cmdOpts: { json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const registry = new MaintenanceTaskRegistry();
      const result = await registry.runTask('backup', {
        config: ctx.config,
        configPath: ctx.configPath,
        isJson,
      });

      if (!result.success) {
        process.exitCode = 1;
      }

      if (isJson) {
        outputJson(result);
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (ﺐﺗر)')} » Configuration Backup`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${result.success ? chalk.green('✓') : chalk.red('✗')} ${result.message}\n`);
    });
}

export function registerEnvCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('env')
    .description('Backup all .env files across active projects')
    .option('--json', 'Output env backup results in JSON format')
    .action(async (cmdOpts: { json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const registry = new MaintenanceTaskRegistry();
      const result = await registry.runTask('env', {
        config: ctx.config,
        isJson,
      });

      if (!result.success) {
        process.exitCode = 1;
      }

      if (isJson) {
        outputJson(result);
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (ﺐﺗر)')} » Environment Files Backup`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${result.success ? chalk.green('✓') : chalk.red('✗')} ${result.message}\n`);
    });
}

export function registerGuardCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('guard')
    .description('D drive root guardrail inspection')
    .option('--json', 'Output guard results in JSON format')
    .action(async (cmdOpts: { json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const registry = new MaintenanceTaskRegistry();
      const result = await registry.runTask('guard', {
        config: ctx.config,
        isReportOnly: true,
        isJson,
      });

      if (!result.success) {
        process.exitCode = 1;
      }

      if (isJson) {
        outputJson(result);
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (ﺐﺗر)')} » Root Guardrail`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${result.success ? chalk.green('✓') : chalk.red('✗')} ${result.message}\n`);
    });
}

export function registerMaintenanceCommand(program: Command, getContext: () => CliContext): void {
  program
    .command('maintenance [task]')
    .description('Run all workspace maintenance tasks (backup, env, guard) or a specific task')
    .option('--full', 'Run comprehensive full maintenance pass', false)
    .option('--json', 'Output maintenance results in JSON format')
    .action(async (taskName: string | undefined, cmdOpts: { full?: boolean; json?: boolean }) => {
      const ctx = getContext();
      const isJson = Boolean(cmdOpts.json || ctx.isJson);

      if (!ctx.config) {
        throw new ConfigMissingError('Configuration not loaded');
      }

      const registry = new MaintenanceTaskRegistry();

      if (taskName) {
        const result = await registry.runTask(taskName, {
          config: ctx.config,
          configPath: ctx.configPath,
          isFull: Boolean(cmdOpts.full),
          isJson,
        });

        if (!result.success) {
          process.exitCode = 1;
        }

        if (isJson) {
          outputJson(result);
          return;
        }

        const icon = result.success ? chalk.green('✓') : chalk.red('✗');
        console.log(`\n  ${icon} [${chalk.bold(result.task)}] ${result.message}\n`);
        return;
      }

      const results = await registry.runAll({
        config: ctx.config,
        configPath: ctx.configPath,
        isFull: Boolean(cmdOpts.full),
        isJson,
      });

      const allSuccess = results.every((r) => r.success);
      if (!allSuccess) {
        process.exitCode = 1;
      }

      if (isJson) {
        outputJson({ success: allSuccess, results });
        return;
      }

      console.log('');
      console.log(chalk.cyan('═'.repeat(60)));
      console.log(`  ${chalk.bold.cyan('rtb (ﺐﺗر)')} » Workspace Maintenance`);
      console.log(chalk.cyan('═'.repeat(60)));
      console.log('');

      for (const res of results) {
        const icon = res.success ? chalk.green('✓') : chalk.red('✗');
        console.log(`  ${icon} [${chalk.bold(res.task)}] ${res.message}`);
      }
      console.log('');
    });
}

export function registerMaintenanceCommands(program: Command, getContext: () => CliContext): void {
  registerMaintenanceCommand(program, getContext);
  registerBackupCommand(program, getContext);
  registerEnvCommand(program, getContext);
  registerGuardCommand(program, getContext);
}

