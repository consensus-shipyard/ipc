// A cleanup script after npm pack is done.
// This script is plugged into the npm build through package.json scripts.
const fs = require('fs-extra')
const path = require('path')

const contractsDir = path.join(__dirname, '../contracts')

// Make sure the destination directory is empty and exists
fs.remove(contractsDir)

console.log('Cleanup after pack is done.')
