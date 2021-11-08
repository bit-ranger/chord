version: "0.0.1"
let: {
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
  let: {
    arg2: "{{arr let.arg.arg2}}"
  },
  exec: {
    action: "echo",
    args: {
      echo: [
        "hello",
        {
          hello: "world"
        },
        "{{arr arg2}}"
      ]
    }
  },
  assert: """
    (all
      (eq value.0 "hello")
      (eq value.1.hello "world")
      (eq value.2.1 "b")
    )
  """
}