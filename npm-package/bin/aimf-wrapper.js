#!/usr/bin/env node
// Location: npm-package/bin/aimf-wrapper.js

const { spawn } = require('child_process');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Try to find aimf binary
function findAIMF() {
    const commonPaths = [
        '/usr/local/bin/aimf',
        path.join(process.env.HOME, '.cargo/bin/aimf'),
        path.join(process.env.HOME, '.local/bin/aimf')
    ];
    
    // Check PATH
    try {
        const which = execSync('which aimf 2>/dev/null', { encoding: 'utf8' });
        if (which.trim()) return which.trim();
    } catch(e) {}
    
    // Check common paths
    for (const p of commonPaths) {
        if (fs.existsSync(p)) return p;
    }
    
    console.error('❌ AIMF binary not found!');
    console.error('Please install AIMF first:');
    console.error('  cargo install aimf-cli');
    console.error('  or download from: https://github.com/yourusername/aimf');
    process.exit(1);
}

const aimfBin = findAIMF();
const args = process.argv.slice(2);

const child = spawn(aimfBin, args, { stdio: 'inherit' });
child.on('close', (code) => {
    process.exit(code);
});
