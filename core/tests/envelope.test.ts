import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { executeEnvelope, wrapAction } from '../src/utils/envelope.js';
import {
  RtbError,
  ConfigMissingError,
  ProjectNotFoundError,
  DirtyGitError,
} from '../src/errors.js';
import type { CliContext } from '../src/types/context.js';

describe('CommandEnvelope Seam', () => {
  let initialExitCode: number | undefined;

  beforeEach(() => {
    initialExitCode = process.exitCode;
    process.exitCode = 0;
  });

  afterEach(() => {
    process.exitCode = initialExitCode;
    vi.restoreAllMocks();
  });

  it('executes successful action without modifying exitCode or writing error', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    let executed = false;

    await executeEnvelope(async () => {
      executed = true;
    }, { isJson: false });

    expect(executed).toBe(true);
    expect(process.exitCode).toBe(0);
    expect(errorSpy).not.toHaveBeenCalled();
  });

  it('catches RtbError, outputs ANSI error, and sets process.exitCode', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await executeEnvelope(async () => {
      throw new RtbError('Custom error message', 'CUSTOM_CODE', 2);
    }, { isJson: false });

    expect(process.exitCode).toBe(2);
    expect(errorSpy).toHaveBeenCalled();
    expect(errorSpy.mock.calls[0][0]).toContain('Custom error message');
  });

  it('catches domain errors and formats JSON payload with code and message', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await executeEnvelope(async () => {
      throw new ProjectNotFoundError('demo-app');
    }, { isJson: true });

    expect(process.exitCode).toBe(1);
    expect(errorSpy).toHaveBeenCalled();
    const payload = JSON.parse(errorSpy.mock.calls[0][0]);
    expect(payload).toEqual({
      error: true,
      code: 'PROJECT_NOT_FOUND',
      message: "Project 'demo-app' not found.",
    });
  });

  it('formats ConfigMissingError and DirtyGitError correctly in JSON mode', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await executeEnvelope(async () => {
      throw new ConfigMissingError();
    }, { isJson: true });

    let payload = JSON.parse(errorSpy.mock.calls[0][0]);
    expect(payload.code).toBe('CONFIG_MISSING');
    expect(process.exitCode).toBe(1);

    errorSpy.mockClear();

    await executeEnvelope(async () => {
      throw new DirtyGitError();
    }, { isJson: true });

    payload = JSON.parse(errorSpy.mock.calls[0][0]);
    expect(payload.code).toBe('DIRTY_GIT');
  });

  it('catches unknown generic Error and formats INTERNAL_ERROR', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await executeEnvelope(async () => {
      throw new Error('Unexpected database failure');
    }, { isJson: true });

    expect(process.exitCode).toBe(1);
    const payload = JSON.parse(errorSpy.mock.calls[0][0]);
    expect(payload).toEqual({
      error: true,
      code: 'INTERNAL_ERROR',
      message: 'Unexpected database failure',
    });
  });

  it('wrapAction extracts context and options to execute through envelope', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const mockCtx: CliContext = {
      config: null,
      configPath: '',
      isConfigured: true,
      isJson: true,
      isQuiet: false,
      isInteractive: false,
    };

    const actionHandler = wrapAction(() => mockCtx, async (arg1: string) => {
      throw new ProjectNotFoundError(arg1);
    });

    await actionHandler('missing-target');

    expect(process.exitCode).toBe(1);
    expect(errorSpy).toHaveBeenCalled();
    const payload = JSON.parse(errorSpy.mock.calls[0][0]);
    expect(payload.code).toBe('PROJECT_NOT_FOUND');
    expect(payload.message).toContain('missing-target');
  });

  it('wrapAction handles unconfigured context by emitting ConfigMissingError in JSON mode', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const mockCtx: CliContext = {
      config: null,
      configPath: '',
      isConfigured: false,
      isJson: true,
      isQuiet: false,
      isInteractive: false,
    };

    const actionHandler = wrapAction(() => mockCtx, async () => {
      // Should not be reached
    });

    await actionHandler();

    expect(process.exitCode).toBe(1);
    expect(errorSpy).toHaveBeenCalled();
    const payload = JSON.parse(errorSpy.mock.calls[0][0]);
    expect(payload.code).toBe('CONFIG_MISSING');
  });
});
