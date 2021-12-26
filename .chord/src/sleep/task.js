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
    let: {
        duration: "{{case.seconds}}"
    },
    exec: {
        sleep: "{{duration}}"
    },
    assert: `
      (all
        (eq 1 1)
        (eq 2 2)
        (eq 3 3)
      )
    `
}
