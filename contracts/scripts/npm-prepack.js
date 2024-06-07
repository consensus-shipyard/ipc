// A script to place contracts under the conventional contracts/ directory before publishing.
// This script is plugged into the npm build through package.json scripts.
const fs = require('fs-extra')
const path = require('path')

// Define source and destination directories
const srcDir = path.join(__dirname, '../src')
const contractsDir = path.join(__dirname, '../contracts')

// Make sure the destination directory is empty and exists
fs.emptyDirSync(contractsDir)

// Copy from src to contracts
fs.copySync(srcDir, contractsDir, { overwrite: true })

console.log('Preparation for pack is done.')
