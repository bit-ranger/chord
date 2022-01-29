let def = {
    arg: {
        arg1: "123",
        arg2:
            [
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
            concurrency: 10,
            loader: {
                strategy: "fix_size_repeat_last_page"
            },
            step: {}
        }
    }
};
module.exports = () => conf;

let step = conf.stage.smoking.step;


step.step1 = {

    nop: {},
    then: [
        {
            reg: {
                s3_loop_idx: 0
            },
            goto: "s3"
        }
    ]
}

step.s2 = {
    let: {
        s2_goto: "{{reg.s2_goto}}"
    },

    log: "hello world",
    then:
        [
            {
                goto: "{{s2_goto}}"
            }
        ]
}

step.s3 = {
    let: {
        arg2: def.arg.arg2
    },

    echo: [
        "hello",
        {
            hello: "world"
        },
        "{{arr arg2}}"
    ],
    assert:
        `
    (all
      (eq value.0 "hello")
      (eq value.1.hello "world")
      (eq value.2.1 "b")
    )
  `,
    then:
        [
            {
                reg: {
                    s2_goto: "s4"
                },
                goto: "s2"
            }
        ]
}

step.s4 = {
    let: {
        s3_loop_idx: "{{num reg.s3_loop_idx }}"
    },

    nop: {},
    then:
        [
            {
                if: "(lt s3_loop_idx 3)",
                reg: {
                    s3_loop_idx:
                        "{{num (num_add s3_loop_idx 1) }}"
                },
                goto: "s3"
            }
        ]
}