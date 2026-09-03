import type { RtbConfig } from './config.js';

export interface CliContext {
  config: RtbConfig | null;
  configPath: string;
  isConfigured: boolean;
  isJson: boolean;
  isQuiet: boolean;
  isInteractive: boolean;
}
