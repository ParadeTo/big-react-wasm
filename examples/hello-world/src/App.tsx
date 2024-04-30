import {useState} from 'react'


function App() {
    const [num, updateNum] = useState(0);

    const isOdd = num % 2;
    console.log(isOdd)
    return (
        <h3
            onClick={(e) => {
                updateNum(prev => prev + 1);
            }}
        >
            {isOdd ? <div>odd</div> : <p>even</p>}
        </h3>
    );
}

function Child({num}: { num: number }) {
    return <div>{num}</div>;
}

export default App
