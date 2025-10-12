#!/usr/bin/env node
import { Command } from '@commander-js/extra-typings';
import { config as loadEnv } from 'dotenv';

loadEnv();

const program = new Command();
program
  .name('star')
  .description('Keystone Vertex operational CLI')
  .version('0.1.0');

program
  .command('init')
  .description('Bootstrap a Keystone workspace')
  .action(() => {
    console.log('Scaffolding workspace directories...');
    console.log('Configure Anchor.toml, keypairs, and cluster settings.');
  });

const launch = program.command('launch').description('Manage token launches');

launch
  .command('create')
  .description('Create a new launch configuration from JSON definition')
  .option('-c, --config <path>', 'Path to config JSON', 'launch.config.json')
  .option('--cluster <cluster>', 'Cluster endpoint override')
  .action((opts) => {
    console.log(`Validating launch configuration at ${opts.config}`);
    console.log('Dry-running PDA derivations and price curves...');
    if (opts.cluster) {
      console.log(`Using cluster ${opts.cluster}`);
    }
    console.log('Submit Anchor transaction with init_launch instruction.');
  });

launch
  .command('buy')
  .description('Participate in launch sale')
  .requiredOption('-a, --amount <number>', 'Token amount to purchase')
  .requiredOption('-m, --max-quote <number>', 'Maximum quote tokens permitted')
  .option('--proof <path>', 'Path to whitelist proof JSON')
  .action((opts) => {
    console.log(`Building buy instruction for amount ${opts.amount}`);
    console.log(`Max quote enforced at ${opts.maxQuote}`);
    if (opts.proof) {
      console.log(`Loading Merkle proof from ${opts.proof}`);
    }
  });

program
  .command('status')
  .description('Inspect program deployments and PDAs')
  .action(() => {
    console.log('Fetching deployment metadata...');
  });

program.parseAsync().catch((err) => {
  console.error(err);
  process.exit(1);
});
