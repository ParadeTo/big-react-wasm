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
    <Suspense fallback={<div>Out loading</div>}>
      <Child id={1} timeout={1000} />
      <Suspense fallback={<div>Inner loading</div>}>
        <Child id={2} timeout={2000} />
      </Suspense>
    </Suspense>
  )
}

function Child({id, timeout}) {
  const {data} = use(fetchData(id, timeout))

  return (
    <div>
      {id}:{data}
    </div>
  )
}
