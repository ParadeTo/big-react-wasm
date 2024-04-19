import dayjs from 'dayjs'

function App() {
    return (
        <div><Comp>{dayjs().format()}</Comp></div>
    )
}

function Comp({children}) {
    return <span><i>{`Hello world, ${children}`}</i></span>
}

export default App
