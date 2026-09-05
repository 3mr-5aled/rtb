import ora, { type Ora, type Color } from 'ora';
import chalk from 'chalk';
import type { CliContext } from '../types/context.js';

export const RTB_SPINNER_FRAMES = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
export const RTB_SPINNER_INTERVAL = 80;

export interface SpinnerOptions {
  quiet?: boolean;
  json?: boolean;
  silent?: boolean;
  isQuiet?: boolean;
  isJson?: boolean;
  context?: CliContext;
  color?: Color;
  prefixText?: string;
  stream?: NodeJS.WritableStream;
  showTime?: boolean;
}

/**
 * TaskSpinner encapsulates `ora` with custom RTB-themed braille frames,
 * timing statistics, and automatic suppression for non-interactive, CI, quiet, or JSON executions.
 */
export class TaskSpinner {
  private oraInstance: Ora | null = null;
  private startTime: number = 0;
  private isSilent: boolean;
  private text: string;
  private showTime: boolean;
  private spinning: boolean = false;

  constructor(text: string, options?: SpinnerOptions) {
    this.text = text;
    this.showTime = options?.showTime ?? true;

    const isQuiet = Boolean(
      options?.quiet ||
      options?.isQuiet ||
      options?.context?.isQuiet ||
      process.env.RTB_QUIET === '1'
    );
    const isJson = Boolean(
      options?.json ||
      options?.isJson ||
      options?.context?.isJson ||
      process.env.RTB_JSON === '1'
    );
    this.isSilent = Boolean(options?.silent || isQuiet || isJson);

    if (!this.isSilent) {
      const oraConfig: Record<string, any> = {
        text,
        color: options?.color ?? 'yellow',
        prefixText: options?.prefixText ?? '  ',
        spinner: {
          interval: RTB_SPINNER_INTERVAL,
          frames: RTB_SPINNER_FRAMES,
        },
      };
      if (options?.stream) {
        oraConfig.stream = options.stream;
      }
      this.oraInstance = ora(oraConfig);
    }
  }

  public start(text?: string): this {
    if (text) {
      this.text = text;
    }
    this.startTime = Date.now();
    this.spinning = true;

    if (!this.isSilent && this.oraInstance) {
      if (text) {
        this.oraInstance.text = text;
      }
      this.oraInstance.start();
    }
    return this;
  }

  public setText(text: string): this {
    this.text = text;
    if (!this.isSilent && this.oraInstance) {
      this.oraInstance.text = text;
    }
    return this;
  }

  private formatElapsedTime(): string {
    if (!this.showTime || this.startTime === 0) return '';
    const elapsed = Date.now() - this.startTime;
    const timeStr = elapsed >= 1000 ? `${(elapsed / 1000).toFixed(2)}s` : `${elapsed}ms`;
    return chalk.gray(`(${timeStr})`);
  }

  public succeed(text?: string): this {
    this.spinning = false;
    if (this.isSilent || !this.oraInstance) return this;

    const baseText = text ?? this.text;
    const time = this.formatElapsedTime();
    const finalMessage = time ? `${baseText} ${time}` : baseText;

    this.oraInstance.succeed(finalMessage);
    return this;
  }

  public fail(text?: string): this {
    this.spinning = false;
    if (this.isSilent || !this.oraInstance) return this;

    const baseText = text ?? this.text;
    const time = this.formatElapsedTime();
    const finalMessage = time ? `${baseText} ${time}` : baseText;

    this.oraInstance.fail(finalMessage);
    return this;
  }

  public warn(text?: string): this {
    this.spinning = false;
    if (this.isSilent || !this.oraInstance) return this;

    const baseText = text ?? this.text;
    const time = this.formatElapsedTime();
    const finalMessage = time ? `${baseText} ${time}` : baseText;

    this.oraInstance.warn(finalMessage);
    return this;
  }

  public info(text?: string): this {
    this.spinning = false;
    if (this.isSilent || !this.oraInstance) return this;

    const baseText = text ?? this.text;
    const time = this.formatElapsedTime();
    const finalMessage = time ? `${baseText} ${time}` : baseText;

    this.oraInstance.info(finalMessage);
    return this;
  }

  public stop(): this {
    this.spinning = false;
    if (this.isSilent || !this.oraInstance) return this;
    this.oraInstance.stop();
    return this;
  }

  public clear(): this {
    if (this.isSilent || !this.oraInstance) return this;
    this.oraInstance.clear();
    return this;
  }

  public get isSpinning(): boolean {
    return this.spinning;
  }
}

/**
 * Convenient wrapper executing an async task inside an animated task spinner.
 * Safely handles success, warnings, or errors without polluting non-TTY/JSON outputs.
 */
export async function withSpinner<T>(
  label: string,
  taskFn: (spinner: TaskSpinner) => Promise<T> | T,
  options?: SpinnerOptions
): Promise<T> {
  const spinner = new TaskSpinner(label, options);
  spinner.start();
  try {
    const result = await taskFn(spinner);
    if (spinner.isSpinning) {
      spinner.succeed();
    }
    return result;
  } catch (error) {
    if (spinner.isSpinning) {
      const errMsg = error instanceof Error ? error.message : String(error);
      spinner.fail(errMsg);
    }
    throw error;
  }
}
