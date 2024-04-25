import {createRoot} from 'react-dom'
import App from './App.tsx'

const root = createRoot(document.getElementById("root"))
const a = <App/>
console.log(a)
root.render(<App/>)

