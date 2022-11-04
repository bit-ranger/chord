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
            content_type: "text/plain"
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
            boundary: "----content-output----",
            content_type: "application/chord-frame-1.0"
        }
    },

    state: {
        assert: `
        (eq value.0.data.case_args.foo case.foo)
        `
    }
}


