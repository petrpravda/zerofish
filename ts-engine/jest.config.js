// jest.config.js

module.exports = {
    preset: 'ts-jest',
    testEnvironment: 'node',
    testPathIgnorePatterns: ['/node_modules/'],
    moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx'],
    // Use transform instead of globals for ts-jest configuration
    transform: {
        '^.+\\.tsx?$': ['ts-jest', { tsconfig: 'tsconfig.json' }],
    },
    // This will tell Jest to look for tests in the src folder
    roots: ['<rootDir>/src'],
};
