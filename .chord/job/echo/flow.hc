version: "0.0.1"
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
  let: {
    arg2: "{{arr def.arg.arg2}}"
  },
  exec: {
    echo: {
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