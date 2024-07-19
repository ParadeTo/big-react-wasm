import {useState} from 'react'

function Child({num}) {
  console.log('Child Render')
  return <div>Child {num}</div>
}

function Parent() {
  const [num, setNum] = useState(1)
  console.log('Parent render')
  return (
    <div onClick={() => setNum(2)}>
      <Child num={num} />
    </div>
    // <div onClick={() => setNum(2)}>
    //   Parent {num}
    //   <Child num={num} />
    // </div>
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
