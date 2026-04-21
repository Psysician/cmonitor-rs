#!/usr/bin/env node
'use strict';

const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const BIN_NAME = process.platform === 'win32' ? 'cmonitor-rs.exe' : 'cmonitor-rs';
const BIN_PATH = path.join(__dirname, BIN_NAME);

if (!fs.existsSync(BIN_PATH)) {
  console.error('cmonitor-rs binary not found. Try reinstalling: npm install cmonitor-rs');
  process.exit(1);
}

const result = spawnSync(BIN_PATH, process.argv.slice(2), { stdio: 'inherit' });
process.exit(result.status ?? 1);
