const path = require("path")

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
};

module.exports = () => conf;

let prefix = path.join(__dirname, "nodejs-example");
conf.pre.step.install = {
    program: {
        cmd: [
            "npm",
            "--prefix",
            prefix,
            "install"
        ],
        value_to_json: false
    }
}

let step = conf.stage.smoking.step;

step.step1 = {
    let: {
        case: "{{obj case}}"
    },

    program: {
        cmd: [
            "npm",
            "--prefix",
            prefix,
            "run",
            "test",
            "{{obj case}}"
        ],
        value_to_json: true
    },
    assert: `
        (eq value.case_args.foo "bar")
        `
}


