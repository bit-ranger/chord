let conf = {
    version: "0.0.1",
    stage: {
        smoking: {
            step: {}
        }
    }
}
module.exports = () => conf;

let step = conf.stage.smoking.step;

step.step1 = {

    cv: {
        count: {
            init: 10,
            incr: 2
        }
    },

    state: {
        assert: `
        (all
            (gte cv 10)
        )
        `
    }

}