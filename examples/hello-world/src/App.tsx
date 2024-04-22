import {useState} from 'react'

function App() {
    const [a, setA] = useState(() => {
        return "a"
    })
    setTimeout(() => {
        setA('2')
    }, 1000)
    return (
        <div><Comp>{a}</Comp></div>
    )
}

function Comp({children}) {
    return <span><i>{`Hello world, ${children}`}</i></span>
}

export default App
