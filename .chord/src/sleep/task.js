let conf = {
    version: "0.0.1",
    stage: {
        smoking: {
            concurrency: 10,
            loader: {
                strategy: "fix_size_repeat_last_page"
            },
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

    sleep: "{{duration}}",
    assert: `
      (all
        (eq 1 1)
        (eq 2 2)
        (eq 3 3)
      )
    `
}
