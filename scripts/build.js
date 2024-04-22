const {execSync} = require('child_process')
const fs = require('fs')

const cwd = process.cwd()

const isTest = process.argv[2] === '--test'

execSync('rm -rf dist')

execSync(`wasm-pack build packages/react --out-dir ${cwd}/dist/react --out-name jsx-dev-runtime ${isTest ? '--target nodejs' : ''}`)
execSync(`wasm-pack build packages/react --out-dir ${cwd}/dist/react --out-name index ${isTest ? '--target nodejs' : ''}`)
execSync(`wasm-pack build packages/react-dom --out-dir ${cwd}/dist/react-dom --out-name index ${isTest ? '--target nodejs' : ''}`)

// modify react/package.json
const packageJsonFilename = `${cwd}/dist/react/package.json`
const packageJson = JSON.parse(fs.readFileSync(packageJsonFilename).toString("utf-8"))
packageJson.files.push('jsx-dev-runtime.wasm', 'jsx-dev-runtime.js', 'jsx-dev-runtime_bg.js', 'jsx-dev-runtime_bg.wasm')
fs.writeFileSync(packageJsonFilename, JSON.stringify(packageJson))

// modify react-dom/index_bg.js
const reactDomIndexBgFilename = isTest ? `${cwd}/dist/react-dom/index.js` : `${cwd}/dist/react-dom/index_bg.js`
const reactDomIndexBgData = fs.readFileSync(reactDomIndexBgFilename)
fs.writeFileSync(reactDomIndexBgFilename, isTest ? 'const {updateDispatcher}=require("react")' : 'import {updateDispatcher} from "react"\n' + reactDomIndexBgData)


