import {useState} from 'react'

let n = 0

function App() {
    const [name, setName] = useState(() => 'ayou')
    if (n === 0) {
        let tid = setTimeout(() => {
            n++
            setName('ayouayou')
            clearTimeout((tid))
        }, 1000)
    }

    return name
}

// function Comp({children}) {
//     return <span><i>{`Hello world, ${children}`}</i></span>
// }

export default App
