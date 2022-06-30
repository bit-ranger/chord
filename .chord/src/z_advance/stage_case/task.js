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
            loader: {
                name: "stage2_named"
            },
            step: {}
        }
    }
};

module.exports = () => conf;
let smoking = conf.stage.smoking;
let stage2 = conf.stage.stage2;

smoking.step.step1 = {
    var: {
        let: {
            arg2: def.arg.arg2
        }
    },

    value: {
        let: [
            "hello",
            {
                hello: "world"
            },
            "{{arr var.arg2}}"
        ]
    },
    ok: {
        assert: `
      (all
(eq value.0 "hello")
(eq value.1.hello "world")
(eq value.2.1 "b")
)
`
    }
}

smoking.step.step2 = {
    var: {
        let: {
            arg1: def.arg.arg1,
            lon: "{{case.origin_lon}}",
            lat: "{{case.origin_lat}}"
        }
    },

    value: {
        let: "update bas set a = '{{var.lon}}' where b = '{{var.lat}}'"
    },
    ok: {
        assert:
            `
    (all
      (str_start_with var.arg1 "12")
      (str_end_with var.arg1 "23")
      (str_contains var.arg1 "2")
      (eq
        (str_sub var.arg1 1) "23"
      )
      (eq
        (str_sub var.arg1 1 2)
        "2"
      )
    )
  `
    }
}


stage2.step.step3 = {
    var: {
        let: {
            arg1: def.arg.arg1,
            foo: "{{case.foo}}",
            bar: "{{case.bar}}"
        }
    },

    value: {
        let: "update bas set a = '{{var.foo}}' where b = '{{var.bar}}'"
    },
    ok: {
        assert:
            `
    (all
      (str_start_with var.arg1 "12")
      (str_end_with var.arg1 "23")
      (str_contains var.arg1 "2")
      (eq (str_sub var.arg1 1) "23")
      (eq (str_sub var.arg1 1 2) "2")
  )
  `
    }
}