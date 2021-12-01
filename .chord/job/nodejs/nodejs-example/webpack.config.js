const path = require('path')

let baseConfig = {
    entry: {
        example: './src/main.js',
    },
    output: {
        path: path.join(__dirname, 'dist'),
        filename: '[name].js'
    },
    stats: {
        colors: true,
        errorDetails: true
    }
}

module.exports = baseConfig
