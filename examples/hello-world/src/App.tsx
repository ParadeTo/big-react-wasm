import {useState} from 'react'


function App() {
    const [num, updateNum] = useState(0);

    const isOdd = num % 2;

    return (
        <h3
            onClickCapture={(e) => {
                e.stopPropagation()
                console.log('click h3', e.currentTarget)
                updateNum(prev => prev + 1);
            }}
        >
            <div onClick={(e) => {
                console.log('click div', e.currentTarget)
            }}>
                {isOdd ? <div>odd</div> : <p>even</p>}
            </div>

        </h3>
    );
}

function Child({num}: { num: number }) {
    return <div>{num}</div>;
}

export default App
