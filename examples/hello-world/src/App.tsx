import {useState} from 'react'

function App() {
    const [name, setName] = useState(() => 'ayou')
    setTimeout(() => {
        setName('ayouayou')
    }, 1000)
    return (
        <div><Comp>{name}</Comp></div>
    )
}

function Comp({children}) {
    return <span><i>{`Hello world, ${children}`}</i></span>
}

export default App
