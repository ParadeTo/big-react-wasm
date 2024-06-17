import {useState} from 'react'

function App() {
  const [num, updateNum] = useState(0)
  const len = 1

  console.log('num', num)
  return (
    <ul
      onClick={(e) => {
        updateNum((num: number) => num + 1)
      }}>
      {Array(len)
        .fill(1)
        .map((_, i) => {
          return <Child i={`${i} ${num}`} />
        })}
    </ul>
  )
}

function Child({i}) {
  return <p>i am child {i}</p>
}

export default App
