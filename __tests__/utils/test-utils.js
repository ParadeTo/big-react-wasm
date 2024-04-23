const ReactDOM = require('../../dist/react-dom');

exports.renderIntoDocument = (element) => {
    const div = document.createElement('div');
    return ReactDOM.createRoot(div).render(element);
};