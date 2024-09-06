import {Suspense, use} from 'react'

const delay = (t) =>
  new Promise((r) => {
    setTimeout(r, t)
  })

const cachePool: any[] = []

function fetchData(id, timeout) {
  const cache = cachePool[id]
  if (cache) {
    return cache
  }
  return (cachePool[id] = delay(timeout).then(() => {
    return {data: Math.random().toFixed(2) * 100}
  }))
}

export default function App() {
  return (
    <Suspense fallback={<div>loading</div>}>
      <Child />
    </Suspense>
  )
}

function Child() {
  const {data} = use(fetchData(1, 1000))

  return <span>{data}</span>
}
