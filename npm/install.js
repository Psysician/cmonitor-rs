#!/usr/bin/env node
'use strict';

const { execFileSync, spawnSync } = require('child_process');
const fs = require('fs');
const https = require('https');
const path = require('path');
const os = require('os');

const REPO = 'Psysician/cmonitor-rs';
const BIN_DIR = path.join(__dirname, 'bin');
const WIN = process.platform === 'win32';
const BIN_NAME = WIN ? 'cmonitor-rs.exe' : 'cmonitor-rs';
const BIN_PATH = path.join(BIN_DIR, BIN_NAME);

function artifactName(version) {
  const arch = process.arch === 'arm64' ? 'arm64' : 'amd64';
  const platform = { darwin: 'darwin', linux: 'linux', win32: 'windows' }[process.platform];
  if (!platform) throw new Error(`Unsupported platform: ${process.platform}`);
  const ext = WIN ? 'zip' : 'tar.gz';
  return { name: `cmonitor-rs-${platform}-${arch}`, ext };
}

function downloadTo(url, dest) {
  return new Promise((resolve, reject) => {
    function get(u) {
      https.get(u, { headers: { 'User-Agent': 'cmonitor-rs-npm-installer' } }, res => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          get(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`HTTP ${res.statusCode} for ${u}`));
          return;
        }
        const out = fs.createWriteStream(dest);
        res.pipe(out);
        out.on('finish', resolve);
        out.on('error', reject);
      }).on('error', reject);
    }
    get(url);
  });
}

async function install() {
  const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, 'package.json'), 'utf8'));
  const { name, ext } = artifactName(pkg.version);
  const artifact = `${name}.${ext}`;
  const url = `https://github.com/${REPO}/releases/download/v${pkg.version}/${artifact}`;
  const tmp = path.join(os.tmpdir(), artifact);

  fs.mkdirSync(BIN_DIR, { recursive: true });
  console.log(`cmonitor-rs: downloading ${artifact}...`);

  await downloadTo(url, tmp);

  if (WIN) {
    execFileSync('powershell', ['-Command',
      `Expand-Archive -Path '${tmp}' -DestinationPath '${BIN_DIR}' -Force`
    ]);
  } else {
    spawnSync('tar', ['-xzf', tmp, '-C', BIN_DIR], { stdio: 'inherit' });
  }

  fs.unlinkSync(tmp);

  if (!fs.existsSync(BIN_PATH)) {
    throw new Error('Binary not found after extraction.');
  }
  fs.chmodSync(BIN_PATH, 0o755);
  console.log('cmonitor-rs: installed.');
}

install().catch(err => {
  console.error('cmonitor-rs install failed:', err.message);
  process.exit(1);
});
