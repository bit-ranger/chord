let def = {
    arg: {
        arg1: "123",
        arg2: [
            "a",
            "b",
            "c"
        ]
    }
}


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
        arg2: def.arg.arg2
    },
    exec: {
        echo: [
            "hello",
            {
                hello: "world"
            },
            "{{arr arg2}}"
        ]
    },
    assert: `
      (all
        (eq value.0 "hello")
        (eq value.1.hello "world")
        (eq value.2.1 "b")
    )
    `
}
