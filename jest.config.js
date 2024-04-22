const {defaults} = require('jest-config');

module.exports = {
    ...defaults,
    moduleDirectories: [...defaults.moduleDirectories, 'dist'],
    testEnvironment: 'jsdom'
};
