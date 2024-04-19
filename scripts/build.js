const {execSync} = require('child_process')
const fs = require('fs')
const path = require('path')

const cwd = process.cwd()

execSync(`wasm-pack build packages/react --out-dir ${cwd}/dist/react --out-name jsx-dev-runtime`)
execSync(`wasm-pack build packages/react --out-dir ${cwd}/dist/react --out-name index`)
execSync(`wasm-pack build packages/react-dom --out-dir ${cwd}/dist/react-dom --out-name index`)


const packageJsonFilename = `${cwd}/dist/react/package.json`
const packageJson = JSON.parse(fs.readFileSync(packageJsonFilename).toString("utf-8"))

packageJson.files.push('jsx-dev-runtime.wasm', 'jsx-dev-runtime.js', 'jsx-dev-runtime_bg.js', 'jsx-dev-runtime_bg.wasm')

fs.writeFileSync(packageJsonFilename, JSON.stringify(packageJson))

