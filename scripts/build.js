const {execSync} = require('child_process')
const fs = require('fs')

const cwd = process.cwd()

execSync(`wasm-pack build packages/react --out-dir ${cwd}/dist/react --out-name jsx-dev-runtime`)
execSync(`wasm-pack build packages/react --out-dir ${cwd}/dist/react --out-name index`)
execSync(`wasm-pack build packages/react-dom --out-dir ${cwd}/dist/react-dom --out-name index`)

// modify react/package.json
const packageJsonFilename = `${cwd}/dist/react/package.json`
const packageJson = JSON.parse(fs.readFileSync(packageJsonFilename).toString("utf-8"))
packageJson.files.push('jsx-dev-runtime.wasm', 'jsx-dev-runtime.js', 'jsx-dev-runtime_bg.js', 'jsx-dev-runtime_bg.wasm')
fs.writeFileSync(packageJsonFilename, JSON.stringify(packageJson))

// modify react-dom/index_bg.js
const reactDomIndexBgFilename = `${cwd}/dist/react-dom/index_bg.js`
const reactDomIndexBgData = fs.readFileSync(reactDomIndexBgFilename)
fs.writeFileSync(reactDomIndexBgFilename, 'import {updateDispatcher} from "react"\n' + reactDomIndexBgData)


