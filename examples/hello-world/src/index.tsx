import {createRoot} from 'react-dom'
import App from './App.tsx'
import './index.css'
import dayjs from "dayjs";

console.log(dayjs())
// @ts-ignore
const a = <App key='appkey' name='app' children={<span>b</span>} ref='b'/>
console.log(a)
const root = createRoot(document.getElementById('root')!)
//@ts-ignore
root.render(a)
// .render(
//     <React.StrictMode>
//         <App/>
//     </React.StrictMode>,
// )
