import {useState, useEffect} from 'react'
// function App() {
//   const [num, updateNum] = useState(0)
//   const len = 100

//   console.log('num', num)
//   return (
//     <ul
//       onClick={(e) => {
//         updateNum((num: number) => num + 1)
//       }}>
//       {Array(len)
//         .fill(1)
//         .map((_, i) => {
//           return <Child i={`${i} ${num}`} />
//         })}
//     </ul>
//   )
// }

// function Child({i}) {
//   return <p>i am child {i}</p>
// }

// export default App

const Item = ({i, children}) => {
  for (let i = 0; i < 999999; i++) {}
  return <span key={i}>{children}</span>
}

export default () => {
  const [count, updateCount] = useState(0)

  const onClick = () => {
    updateCount(2)
  }

  useEffect(() => {
    const button = document.querySelector('button')
    setTimeout(() => updateCount((num) => num + 1), 1000)
    setTimeout(() => button.click(), 1100)
  }, [])

  return (
    <div>
      <button onClick={onClick}>增加2</button>
      <div style={{wordWrap: 'break-word'}}>
        {Array.from(new Array(1000)).map((v, index) => (
          <Item i={index}>{count}</Item>
        ))}
      </div>
    </div>
  )
}
