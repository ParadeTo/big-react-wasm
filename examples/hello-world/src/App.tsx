import {useState} from 'react'

function App() {
    const [num, updateNum] = useState(0);

    const isOdd = num % 2 === 1;

    const before = [
        <li key={1}>1</li>,
        <li>2</li>,
        <li>3</li>,
        <li key={4}>4</li>
    ];
    const after = [
        <li key={4}>4</li>,
        <li>2</li>,
        <li>3</li>,
        <li key={1}>1</li>
    ];

    const listToUse = isOdd ? after : before;
    console.log(num, listToUse)
    return (
        <ul
            onClick={(e) => {
                updateNum(num => num + 1);
            }}
        >
            {listToUse}
        </ul>
    );
}

function Child({num}: { num: number }) {
    return <div>{num}</div>;
}

export default App
