import {Suspense, use} from 'react'

export default function App() {
  return (
    <Suspense fallback={<div>loading</div>}>
      <Child />
    </Suspense>
  )
}

function Child() {
  try {
    use(new Promise((resolve) => setTimeout(resolve, 1000)))
  } catch (error) {
    debugger
  }
}
