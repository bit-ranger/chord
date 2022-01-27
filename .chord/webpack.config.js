const JsonFromJsPlugin = require("./JsonFromJsPlugin");
const path = require("path");
const CopyWebpackPlugin = require('copy-webpack-plugin')

let allTasks = [
    "cdylib",
    "count",
    "database",
    "docker",
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

module.exports = (env) => {
    console.log(env)
    let enableTasks = allTasks;
    if (env["task"]) {
        let task = env["task"];
        enableTasks = allTasks.filter(t => t === task);
    }

    let plugins = [
        new CopyWebpackPlugin({
            patterns: enableTasks.map(t => {
                    return {
                        from: `./${t}/*.csv`
                    }
                }
            )
        }),
        new CopyWebpackPlugin({
            patterns: [
                {
                    from: `./**/chord.conf`
                }
            ]
        }),
        new JsonFromJsPlugin({
            patterns: enableTasks.map(e => {
                    return {
                        from: `./${e}/task.js`,
                        to: `${e}/task.conf`
                    }
                }
            )
        })
    ]

    let entry = {}
    enableTasks.map(t => {
        entry[t] = `./${t}/task.js`
    });


    return {
        target: "node",
        context: path.join(__dirname, './src'),
        entry: entry,
        output: {
            path: path.join(__dirname, './dist'),
            filename: '[name]/task.conf',
            clean: true
        },
        plugins: plugins
    }
};