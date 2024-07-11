import {useState} from 'react'

function Child() {
  console.log('Child Render')
  return <div>Child</div>
}

function Parent() {
  const [n, setN] = useState(0)
  console.log('Parent render')
  return (
    <div onClick={() => setN(1)}>
      Parent
      <Child />
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
