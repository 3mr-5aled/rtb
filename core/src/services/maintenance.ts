import fs from 'node:fs';
import path from 'node:path';
import type { RtbConfig } from '../types/config.js';

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
