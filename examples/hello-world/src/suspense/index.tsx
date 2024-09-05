import {Suspense} from 'react'

export default function App() {
  return (
    <Suspense fallback={<div>loading</div>}>
      <Child />
    </Suspense>
  )
}

function Child() {
  try {
    throw new Promise((resolve) => setTimeout(resolve, 1000))
  } catch (error) {
    debugger
  }
}
