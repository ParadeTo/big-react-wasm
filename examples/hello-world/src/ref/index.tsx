import {useState, useEffect, useRef} from 'react'

export default function App() {
  const [isDel, del] = useState(false)
  const divRef = useRef(null)

  console.warn('render divRef', divRef.current)

  useEffect(() => {
    console.warn('useEffect divRef', divRef.current)
  }, [])

  return (
    <div ref={divRef} onClick={() => del((prev) => !prev)}>
      {isDel ? null : <Child />}
    </div>
  )
}

function Child() {
  return <p ref={(dom) => console.warn('dom is:', dom)}>Child</p>
}
