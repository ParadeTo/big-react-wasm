import {useState} from 'react'

export default function App() {
  const [num, setNum] = useState(100)
  const arr =
    num % 2 === 0
      ? [<li key='1'>1</li>, <li key='2'>2</li>, <li key='3'>3</li>]
      : [<li key='3'>3</li>, <li key='2'>2</li>, <li key='1'>1</li>]
  return (
    <ul onClick={() => setNum((num) => num + 1)}>
      {/* <li>4</li>
      <li>5</li> */}
      {arr}
      <span>{num}</span>

      {/* {num} */}
    </ul>
  )
}
