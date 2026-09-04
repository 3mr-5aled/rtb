import type { CliContext } from '../types/context.js';
import { RtbError, ConfigMissingError } from '../errors.js';
import { outputError } from './output.js';

export interface EnvelopeOptions {
  isJson?: boolean;
}

export async function executeEnvelope<T>(
  action: () => Promise<T> | T,
  options: EnvelopeOptions = {}
): Promise<T | void> {
  try {
    return await action();
  } catch (err: unknown) {
    if (err instanceof RtbError || (err instanceof Error && 'code' in err)) {
      const code = (err as any).code || 'RTB_ERROR';
      const exitCode = (err as any).exitCode ?? 1;
      process.exitCode = exitCode;
      outputError(err.message, code, options.isJson);
      return;
    }

    if (err instanceof Error) {
      process.exitCode = 1;
      outputError(err.message, 'INTERNAL_ERROR', options.isJson);
      return;
    }

    process.exitCode = 1;
    outputError(String(err), 'UNKNOWN_ERROR', options.isJson);
  }
}

export function wrapAction<Args extends any[]>(
  getContext: () => CliContext,
  handler: (...args: Args) => Promise<void> | void,
  options: { exemptFromConfig?: boolean } = {}
): (...args: Args) => Promise<void> {
  return async (...args: Args) => {
    const ctx = getContext();
    if (!options.exemptFromConfig && !ctx.isConfigured) {
      if (process.exitCode === undefined || process.exitCode === 0) {
        await executeEnvelope(() => {
          throw new ConfigMissingError();
        }, { isJson: ctx.isJson });
      }
      return;
    }
    // Check if json option was passed explicitly in command args
    const hasJsonArg = args.some(
      (a) => a && typeof a === 'object' && 'json' in a && Boolean((a as any).json)
    );
    const isJson = Boolean(ctx.isJson || hasJsonArg);

    await executeEnvelope(() => handler(...args), { isJson });
  };
}
