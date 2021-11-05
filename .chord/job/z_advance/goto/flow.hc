version: "0.0.1",
def: {
  arg: {
    arg1: "123",
    arg2: [
      "a",
      "b",
      "c"
    ]
  }
}


stage.s1.step.s1: {
  exec: {
    action: "nop",
    args: {}
  },
  then: [
    {
      reg: {
        s3_loop_idx: 0
      },
      goto: "s3"
    }
  ]
}

stage.s1.step.s2: {
  let: {
    s2_goto: "{{reg.s2_goto}}"
  },
  exec: {
    action: "log",
    args: {
      log: "hello world"
    }
  },
  then: [
    {
      goto: "{{s2_goto}}"
    }
  ]
}

stage.s1.step.s3: {
  let: {
    arg2: "{{ref def.arg.arg2}}"
  },
  exec: {
    action: "echo",
    args: {
      echo: [
        "hello",
        {
          hello: "world"
        },
        "{{ref arg2}}"
      ]
    }
  },
  assert: """
    (all
      (eq value.0 "hello")
      (eq value.1.hello "world")
      (eq value.2.1 "b")
    )
  """,
  then: [
    {
      reg: {
        s2_goto: "s4"
      },
      goto: "s2"
    }
  ]
}

stage.s1.step.s4: {
  let: {
    s3_loop_idx: "{{num {{reg.s3_loop_idx}}}}"
  },
  exec: {
    action: "nop",
    args: {}
  },
  then: [
    {
      if: "(lt s3_loop_idx 3)",
      reg: {
        s3_loop_idx:
          "{{num {{num_add s3_loop_idx 1}}}}"
      },
      goto: "s3"
    }
  ]
}