const path = require("path")

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

let prefix = path.join(__dirname, "nodejs-example");

step.step1 = {

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


step.step2 = {
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


