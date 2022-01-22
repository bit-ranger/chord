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

    count: {
        init: 10,
        incr: 2
    },
    assert: `
      (all
        (gt value 9)
      )
    `
}