import {useState} from 'react'


function App() {
    const [name, setName] = useState(() => 'ayou')
    let tid = setTimeout(() => {
        setName('ayouayou')
        clearTimeout((tid))
    }, 1000)
    return name
}

// function Comp({children}) {
//     return <span><i>{`Hello world, ${children}`}</i></span>
// }

export default App
