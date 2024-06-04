const ReactNoop = require('react-noop')
const React = require('react')

const root = ReactNoop.createRoot()
const useEffect = React.useEffect

function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms)
  })
}

async function run() {
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

  root.render(<Parent a={1} />)
  await sleep(1000)
  console.log(root.getChildrenAsJSX())
}

run()
