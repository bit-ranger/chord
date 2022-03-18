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
    var: {
        let: {
            x: 1
        }
    },

    value: {
        match: {
            "(eq var.x 2)": {
                l: {
                    log: "var.x eq 2"
                },
                v: {
                    let: 2
                }
            },

            "(eq var.x 1)": {
                l: {
                    log: "var.x eq 1"
                },
                v: {
                    let: 1
                }
            }
        }
    },
    ok: {
        assert: `
        (eq value.v 1)
        `
    }
}