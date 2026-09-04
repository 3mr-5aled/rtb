import chalk from 'chalk';

/**
 * Pre-shaped joined Arabic glyphs in visual LTR order (ﺐ + ﺗ + ر)
 * Ensures Arabic brand renders properly connected as "رتّب" across LTR terminals.
 */
export const RTB_BRAND_ARABIC = '\uFE90\uFE97\u0631'; // ﺐﺗر

export function outputJson(data: unknown): void {
  console.log(JSON.stringify(data, null, 2));
}

export function outputError(message: string, code?: string, isJson?: boolean): void {
  if (isJson) {
    console.error(
      JSON.stringify(
        {
          error: true,
          message,
          ...(code ? { code } : {}),
        },
        null,
        2
      )
    );
  } else {
    console.error(`  ${chalk.red('✗')} ${chalk.red(message)}`);
  }
}
