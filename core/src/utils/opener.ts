import { spawn } from 'node:child_process';
import os from 'node:os';

export type LauncherFn = (cmd: string, args: string[]) => void;

export function resolveOpenCommand(platform: string = process.platform): string {
  if (platform === 'win32') {
    return 'explorer.exe';
  } else if (platform === 'darwin') {
    return 'open';
  } else {
    return 'xdg-open';
  }
}

export function defaultLauncher(cmd: string, args: string[]): void {
  const child = spawn(cmd, args, {
    detached: true,
    stdio: 'ignore',
  });
  child.unref();
}

export function openPath(
  targetPath: string,
  launcher: LauncherFn = defaultLauncher,
  platform: string = process.platform
): void {
  const cmd = resolveOpenCommand(platform);
  launcher(cmd, [targetPath]);
}
