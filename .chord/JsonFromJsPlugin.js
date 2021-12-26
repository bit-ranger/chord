const path = require('path')
const {Compilation, sources} = require('webpack');

function JsonFromJsPlugin(config = {}) {
    Object.assign(this, {
        ...config,
        patterns: config.patterns
    })
}

JsonFromJsPlugin.prototype.apply = function apply(compiler) {

    if (compiler.hooks) {
        compiler.hooks.thisCompilation.tap("JsonFromJsPlugin", (compilation) => {
            compilation.hooks.processAssets.tap(
                {
                    name: "JsonFromJsPlugin",
                    stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE,
                },
                () => {
                    for (const pattern of this.patterns) {
                        const fullFilePath = path.resolve(compiler.context, pattern.from)

                        const jsModule = require(fullFilePath)
                        let jsonValue = null

                        jsonValue = jsModule(this.data)

                        if (jsonValue) {
                            const json = JSON.stringify(jsonValue, null, 2)
                            compilation.updateAsset(
                                pattern.to,
                                new sources.RawSource(json)
                            )
                        }
                    }


                })
        });
    }
}

module.exports = JsonFromJsPlugin