export class RtbError extends Error {
  public readonly isRtbError = true;

  constructor(
    message: string,
    public readonly code: string = 'RTB_ERROR',
    public readonly exitCode: number = 1
  ) {
    super(message);
    this.name = 'RtbError';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

export class ConfigMissingError extends RtbError {
  constructor(message = 'RTB is not configured yet. Run "rtb init" to initialize workspace.') {
    super(message, 'CONFIG_MISSING', 1);
    this.name = 'ConfigMissingError';
  }
}

export class ConfigInvalidError extends RtbError {
  constructor(message: string) {
    super(message, 'CONFIG_INVALID', 1);
    this.name = 'ConfigInvalidError';
  }
}

export class ProjectNotFoundError extends RtbError {
  constructor(messageOrName?: string, code = 'PROJECT_NOT_FOUND') {
    const message = messageOrName
      ? messageOrName.startsWith('Project')
        ? messageOrName
        : `Project '${messageOrName}' not found.`
      : 'Project not found.';
    super(message, code, 1);
    this.name = 'ProjectNotFoundError';
  }
}

export class DirtyGitError extends RtbError {
  constructor(message = 'Working tree has uncommitted changes.') {
    super(message, 'DIRTY_GIT', 1);
    this.name = 'DirtyGitError';
  }
}
