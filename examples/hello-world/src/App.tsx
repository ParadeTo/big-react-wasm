import {useEffect, useState} from 'react'

function App() {
    const [num, updateNum] = useState(0);
    return (
        <ul
            onClick={(e) => {
                updateNum((num: number) => num + 1);
            }}
        >
            <Child1 num={num}/>
            {num === 1 ? null : <Child2 num={num}/>}
        </ul>
    );
}

function Child1({num}: { num: number }) {
    useEffect(() => {
        console.log('child1 create')
        return () => {
            console.log('child1 destroy')
        }
    }, [num]);
    return <div>child1 {num}</div>;
}

function Child2({num}: { num: number }) {
    useEffect(() => {
        console.log('child2 create')
        return () => {
            console.log('child2 destroy')
        }
    }, [num]);
    return <div>child2 {num}</div>;
}

export default App
