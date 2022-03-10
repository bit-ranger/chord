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
let pre = conf.pre;
let smoking = conf.stage.smoking;


pre.step.p1 = {

    value: {
        program: {
            cmd: [
                "python3",
                "--version"
            ]
        }
    }
}

pre.step.p2 = {

    value: {
        program: {
            detach: true,
            cmd: [
                "python3",
                "--version"
            ]
        }
    }
}

smoking.step.step1 = {

    value: {
        program: {
            cmd: [
                "python3",
                "--version"
            ]
        }
    }
}

smoking.step.step2 = {

    value: {
        program: {
            detach: true,
            cmd: [
                "python3",
                "--version"
            ]
        }
    }
}

smoking.step.s3 = {

    value: {
        program: {
            cmd: [
                "python3",
                "--version"
            ]
        }
    }
}