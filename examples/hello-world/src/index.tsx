import {jsxDEV} from 'react'
import {createRoot}  from 'react-dom'
import App from './App.tsx'
import './index.css'

// @ts-ignore
const a = <App key='appkey' name='app' children={<span>b</span>} ref='b' />

const root = createRoot(document.getElementById('root')!)
//@ts-ignore
// root.render(<App />)
//     .render(
//     <React.StrictMode>
//         <App />
//     </React.StrictMode>,
// )
