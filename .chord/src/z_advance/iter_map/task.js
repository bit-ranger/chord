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
    exec: {
        iter_map: {
            arr: [
                "a",
                "b",
                "c"
            ],
            exec: {
                echo: [
                    {
                        idx: "0-{{idx}}",
                        item: "0-{{item}}"
                    },
                    {
                        idx: "1-{{idx}}",
                        item: "1-{{item}}"
                    }
                ]
            }
        }
    },
    assert: `
    (all
      (eq state "Ok")
      (eq value.2.1.item "1-c")
    )
  `
}


step.step2 = {
    let: {
        content: "{{arr step.step1.value}}"
    },
    exec: {
        log: "{{arr content}}"
    }
}