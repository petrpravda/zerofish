// jest.config.js

module.exports = {
    preset: 'ts-jest',
    testEnvironment: 'node',
    testPathIgnorePatterns: [
        '/node_modules/', // Default to ignore node_modules
        '/src/__tests__/testing.perft.helper.ts',
        '/src/__tests__/testing.helper.ts',
    ],    moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx'],
    // Use transform instead of globals for ts-jest configuration
    transform: {
        '^.+\\.tsx?$': ['ts-jest', { tsconfig: 'tsconfig.json' }],
    },
    // This will tell Jest to look for tests in the src folder
    roots: ['<rootDir>/src'],
};
