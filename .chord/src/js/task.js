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
    value: {
        // language=js
        js: `
            console.log(chordVal)
        `,
    },

    state: {
        assert: `
        (all 
            (eq (str value.1.bar) var.bar)
        )
        `
    }
}
