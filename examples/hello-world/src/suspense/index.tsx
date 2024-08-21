import {Suspense} from 'react'

export default function App() {
  return (
    <Suspense fallback={<div>loading</div>}>
      <Child />
    </Suspense>
  )
}

function Child() {
  debugger
  throw Promise.resolve(1)
  return <p>Child</p>
}
