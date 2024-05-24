import {useEffect, useState} from 'react'

function App() {
    const [num, updateNum] = useState(0);
    useEffect(() => {
        console.log(num)
        setTimeout(() => {
            updateNum(num + 1)
        }, 1000)
    }, [num]);
    return (
        <ul
            onClick={(e) => {
                // 注意观察多次更新只会触发一次render阶段，这就是batchedUpdates（批处理），也是我们基础调度能力的体现
                updateNum((num: number) => num + 1);
                updateNum((num: number) => num + 2);
                updateNum((num: number) => num + 3);
                updateNum((num: number) => num + 4);
            }}
        >
            <Child num={num}/>
            hello
            <Child num={num}/>
        </ul>
    );
}

function Child({num}: { num: number }) {

    return <div>{num}</div>;
}

export default App
