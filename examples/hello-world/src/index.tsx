import {createRoot} from 'react-dom'
import './index.css'
import dayjs from "dayjs";

console.log(dayjs())
// @ts-ignore
const a = <p>hello world</p>
console.log(a)
const root = createRoot(document.getElementById('root')!)

//@ts-ignore
root.render(a)
// .render(
//     <React.StrictMode>
//         <App/>
//     </React.StrictMode>,
// )
