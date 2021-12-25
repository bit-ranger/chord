const JsToJsonPlugin = require("./JsToJsonPlugin");
const path = require("path");
const CopyWebpackPlugin = require('copy-webpack-plugin');


let tasks = [
    "lua",
    "restapi"
]

let entry = {}
tasks.map(t => {
    entry[t] = `./${t}/task.js`
});


let plugins = [
    new CopyWebpackPlugin({
        patterns: tasks.map(t => {
                return {
                    from: `./${t}/*.csv`,
                    to: `./${t}/[name].csv`
                }
            }
        )
    }),
    new JsToJsonPlugin({
        patterns: tasks.map(e => {
                return {
                    from: `./${e}/task.js`,
                    to: `${e}/task.conf`
                }
            }
        )
    })
]


module.exports = {
    target: "node",
    context: path.join(__dirname, './src'),
    entry: entry,
    output: {
        path: path.join(__dirname, './dist'),
        filename: '[name]/task.conf',
    },
    plugins: plugins
};