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

    iter_map: {
        iter: [
            "a",
            "b",
            "c"
        ],
        map: {
            echo: [
                {
                    x: "0-{{idx}}",
                    y: "0-{{item}}"
                },
                {
                    x: "1-{{idx}}",
                    y: "1-{{item}}"
                }
            ]
        }
    },
    assert:
        `
    (all
      (eq state "Ok")
      (eq value.2.1.y "1-c")
    )
  `
}


step.step2 = {
    let: {
        content: "{{arr step.step1.value}}"
    },

    log: "{{arr content}}"
}
