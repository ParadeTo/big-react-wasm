import {Suspense, use} from 'react'

export default function App() {
  return (
    <Suspense fallback={<div>loading</div>}>
      <Child />
    </Suspense>
  )
}

function Child() {
  const a = use(new Promise((resolve) => setTimeout(() => resolve(1), 1000)))
  return <span>{a}</span>
}
