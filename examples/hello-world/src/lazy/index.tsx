import {Suspense, lazy} from 'react'

function delay(promise) {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve(promise)
    }, 2000)
  })
}

const Cpn = lazy(() => import('./Cpn').then((res) => delay(res)))

export default function App() {
  return (
    <Suspense fallback={<div>loading</div>}>
      <Cpn />
    </Suspense>
  )
}
