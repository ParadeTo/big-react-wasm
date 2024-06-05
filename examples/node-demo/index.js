const ReactNoop = require('react-noop')
const React = require('react')

const root = ReactNoop.createRoot()
const useEffect = React.useEffect

function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms)
  })
}

async function test1() {
  const arr = []

  function Parent() {
    useEffect(() => {
      return () => arr.push('Unmount parent')
    })
    return <Child />
  }

  function Child() {
    useEffect(() => {
      return () => arr.push('Unmount child')
    })
    return 'Child'
  }

  root.render(<Parent a={1} />)
  await sleep(10)
  if (root.getChildrenAsJSX() !== 'Child') {
    throw new Error('test1 failed')
  }

  root.render(<div>a</div>)
  await sleep(10)
  console.log(root.getChildrenAsJSX())
  // if (root.getChildrenAsJSX() !== null) {
  //   throw new Error('test1 failed')
  // }
}

test1()
