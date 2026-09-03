import chalk from 'chalk';

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
