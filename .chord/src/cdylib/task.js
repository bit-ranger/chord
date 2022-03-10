let conf = {
    version: "0.0.1",
    stage: {
        smoking: {
            step: {}
        }
    }
};

module.exports = () => conf;

let step = conf.stage.smoking.step;

step.step1 = {
    value: {
        cdylib: "cdylib_example",
    },
    state: {
        assert: `
            (all
                (eq value.run 1)
            )
        `
    }
}

