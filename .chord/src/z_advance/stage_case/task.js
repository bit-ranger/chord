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
            step: {}
        },
        stage2: {
            step: {}
        }
    }
};

module.exports = () => conf;
let smoking = conf.stage.smoking;
let stage2 = conf.stage.stage2;

smoking.step.step1 = {
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
    assert: `
      (all
(eq value.0 "hello")
(eq value.1.hello "world")
(eq value.2.1 "b")
)
`
}

smoking.step.step2 = {
    let: {
        arg1: def.arg.arg1,
        lon: "{{case.origin_lon}}",
        lat: "{{case.origin_lat}}"
    },

    echo: "update bas set a = '{{lon}}' where b = '{{lat}}'",
    assert:
        `
    (all
      (str_start_with arg1 "12")
      (str_end_with arg1 "23")
      (str_contains arg1 "2")
      (eq
        (str_sub arg1 1) "23"
      )
      (eq
        (str_sub arg1 1 2)
        "2"
      )
    )
  `
}


stage2.step.step3 = {
    let: {
        arg1: def.arg.arg1,
        foo: "{{case.foo}}",
        bar: "{{case.bar}}"
    },

    echo: "update bas set a = '{{foo}}' where b = '{{bar}}'",
    assert:
        `
    (all
      (str_start_with arg1 "12")
      (str_end_with arg1 "23")
      (str_contains arg1 "2")
      (eq (str_sub arg1 1) "23")
      (eq (str_sub arg1 1 2) "2")
  )
  `
}