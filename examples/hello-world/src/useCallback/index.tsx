import {useState, useCallback} from 'react'

let lastFn

export default function App() {
  const [num, update] = useState(1)
  console.log('App render ', num)

  const addOne = useCallback(() => update((n) => n + 1), [])
  // const addOne = () => update((n) => n + 1)

  if (lastFn === addOne) {
    console.log('useCallback work')
  }

  lastFn = addOne

  return (
    <div>
      <Cpn onClick={addOne} />
      {num}
    </div>
  )
}

const Cpn = function ({onClick}) {
  console.log('Cpn render')
  return <div onClick={() => onClick()}>lll</div>
}
