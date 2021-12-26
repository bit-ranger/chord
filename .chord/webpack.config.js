const JsonFromJsPlugin = require("./JsonFromJsPlugin");
const path = require("path");


let tasks = [
    "cdylib",
    "count",
    "database",
    "docker",
    "download",
    "dubbo",
    "echo",
    "helper",
    "log",
    "lua",
    "nodejs",
    "program",
    "redis",
    "restapi",
    "sleep",
    "typecast",
    "z_advance/gen_dml",
    "z_advance/goto",
    "z_advance/iter_map",
    "z_advance/stage_case"
]

let entry = {}
tasks.map(t => {
    entry[t] = `./${t}/task.js`
});


let plugins = [
    new JsonFromJsPlugin({
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
        path: path.join(__dirname, './src'),
        filename: '[name]/task.conf',
    },
    plugins: plugins
};