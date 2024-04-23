import {createRoot} from 'react-dom'

const comp = <div><p><span>Hello World</span></p></div>
const root = createRoot(document.getElementById("root"))
root.render(comp)

