import {useState} from 'react'

function Child({num}) {
  console.log('Child Render')
  return <div>Child {num}</div>
}

function Parent() {
  const [num, setNum] = useState(0)
  console.log('Parent render')
  return (
    <div onClick={() => setNum(1)}>
      Parent {num}
      <Child num={num} />
    </div>
  )
}

export default function App() {
  console.log('App render')
  return (
    <div>
      App
      <Parent />
    </div>
  )
}
