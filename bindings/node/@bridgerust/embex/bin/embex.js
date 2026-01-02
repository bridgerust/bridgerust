#!/usr/bin/env node

const { cli } = require("../index.js");

const args = process.argv.slice(1);
// process.argv is [node, script, args...]
// We want [script, args...] so Clap treats 'script' as the binary name and args as args.

cli(args).catch((err) => {
  console.error(err);
  process.exit(1);
});
