import {createRoot} from 'react-dom'
import {useEffect} from 'react'

const root = createRoot(document.getElementById('root'))

function Parent() {
  useEffect(() => {
    return () => console.log('Unmount parent')
  })
  return <Child />
}

function Child() {
  useEffect(() => {
    return () => console.log('Unmount child')
  })
  return 'Child'
}

root.render(<Parent />)
// console.log(root.getChildrenAsJSX())
