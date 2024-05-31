let React;
let ReactNoop;
let act;
let useEffect;

function sleep(ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms)
    })
}

describe('ReactHooksWithNoopRenderer', () => {
    beforeEach(() => {
        jest.resetModules();

        React = require('../../dist/react');
        act = require('jest-react').act;
        // Scheduler = require('scheduler/unstable_mock');
        ReactNoop = require('../../dist/react-noop');

        useEffect = React.useEffect;
    });

    it.only('passive unmounts on deletion are fired in parent -> child order', async () => {

        const root = ReactNoop.createRoot();

        function Parent() {
            useEffect(() => {
                return () => console.log('Unmount parent');
            });
            return <Child/>;
        }

        function Child() {
            useEffect(() => {
                return () => console.log('Unmount child');
            });
            return 'Child';
        }

        root.render(<Parent/>);
        await sleep(10)
        expect(root).toMatchRenderedOutput('Child');
        expect(1).toBe(1)
        // await act(async () => {
        // });
        //
        //
        // await act(async () => {
        //     root.render(null);
        // });

    });
})