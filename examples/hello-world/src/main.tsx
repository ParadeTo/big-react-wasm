import {createRoot} from 'react-dom'
import {useState} from 'react/index'

const [a, setA] = useState();
console.log(a, setA);
const comp = <div><p><span>Hello World</span></p></div>
const root = createRoot(document.getElementById("root"))
root.render(comp)

