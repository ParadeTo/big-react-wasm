import {jsxDEV} from 'react'
import {createRoot}  from 'react-dom'
import App from './App.tsx'
import './index.css'

// @ts-ignore
const a = <App />

createRoot(document.getElementById('root')!)

//     .render(
//     <React.StrictMode>
//         <App />
//     </React.StrictMode>,
// )
