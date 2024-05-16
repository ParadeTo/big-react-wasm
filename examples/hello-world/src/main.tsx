// import App from './App.tsx'
//
// const root = createRoot(document.getElementById("root"))
// root.render(<App/>)
import {Priority, scheduleCallback, shouldYieldToHost} from 'react-dom'


// scheduleCallback(2, function func1() {
//     console.log('1')
// })
//
// const taskId = scheduleCallback(1, function func2() {
//     console.log('2')
// })

// cancelCallback(taskId)


function func2(didTimeout) {
    console.log(didTimeout)
    if (!didTimeout) console.log(2)
}

function func1() {
    console.log(1)
    return func2
}

scheduleCallback(Priority.NormalPriority, func1)

function work() {
    while (!shouldYieldToHost()) {
        console.log('work')
    }
    console.log('yield to host')
}

scheduleCallback(1, function func2() {
    work()
})

