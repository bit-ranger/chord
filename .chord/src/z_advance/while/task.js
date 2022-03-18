let conf = {
    version: "0.0.1",
    stage: {
        smoking: {
            step: {}
        },
    }
};

module.exports = () => conf;
let smoking = conf.stage.smoking;

smoking.step.step1 = {
    cnt: {
        let: 1
    },

    value: {
        while: {
            "(lt cnt 10)": {
                set0: {
                    set: {
                        cnt: "{{num (num_add cnt 1)}}"
                    }
                },
                log1: {
                    log: "{{num cnt}}"
                }
            }
        }
    },
    ok: {
        assert: `
        (eq cnt 10)
        `
    }
}