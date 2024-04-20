import dayjs from 'dayjs'
import {useState} from 'react'

function App() {
    const [a, setA] = useState("a")
    console.log(a, setA('ayou'))
    return (
        <div><Comp>{dayjs().format()}</Comp></div>
    )
}

function Comp({children}) {
    return <span><i>{`Hello world, ${children}`}</i></span>
}

export default App
