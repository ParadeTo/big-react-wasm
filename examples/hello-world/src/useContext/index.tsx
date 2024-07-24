import {useState, createContext, useContext} from 'react'

const ctxA = createContext('deafult A')
const ctxB = createContext('default B')

export default function App() {
  return (
    <ctxA.Provider value={'A0'}>
      <ctxB.Provider value={'B0'}>
        <ctxA.Provider value={'A1'}>
          <Cpn />
        </ctxA.Provider>
      </ctxB.Provider>
      <Cpn />
    </ctxA.Provider>
  )
}

function Cpn() {
  const a = useContext(ctxA)
  const b = useContext(ctxB)
  return (
    <div>
      A: {a} B: {b}
    </div>
  )
}
