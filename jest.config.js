const {defaults} = require('jest-config');

module.exports = {
    ...defaults,
    moduleDirectories: [...defaults.moduleDirectories, 'dist'],
    modulePathIgnorePatterns: ["__tests__/utils"],
    testEnvironment: 'jsdom',
    setupFilesAfterEnv: ['<rootDir>/setup-jest.js'],
};
