import { Command } from 'commander';

const program = new Command();

program
  .name('rtb')
  .description('RTB (رتّب) - Unified workspace & developer project manager')
  .version('0.5.0');

program
  .command('version')
  .description('Display RTB version information')
  .action(() => {
    console.log('rtb version 0.5.0 (core-ts)');
  });

program.parse(process.argv);
