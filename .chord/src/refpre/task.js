let conf = {
    version: "0.0.1",
    pre: {
        step: {}
    },
    stage: {
        smoking: {
            step: {}
        }
    }
}
module.exports = () => conf;

conf.pre.step.init = {
    value: {
        let: "hello"
    }
}

let step = conf.stage.smoking.step;

step.step1 = {

    value: {
        let: "{{pre.step.init.value}}"
    },

    state: {
        assert: `
        (all
            (eq value "hello")
        )
        `
    }

}