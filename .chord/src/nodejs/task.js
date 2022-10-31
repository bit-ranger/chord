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

    value: {
        program: {
            cmd: [
                "npm",
                "--prefix",
                prefix,
                "install"
            ],
            parse_json_str: false
        }
    }

}

let step = conf.stage.smoking.step;

step.step1 = {
    var: {
        let: {
            case: "{{obj case}}"
        }
    },

    value: {
        program: {
            cmd: [
                "npm",
                "--prefix",
                prefix,
                "run",
                "test",
                "{{obj var.case}}"
            ],
            parse_json_str: true
        }
    },

    state: {
        assert: `
        (eq value.case_args.foo "bar")
        `
    }
}


