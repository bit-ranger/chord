version: "0.0.1"


stage.s1.step.s1: {
  exec: {
    action: "iter_map",
    args: {
      arr: [
        "a",
        "b",
        "c"
      ],
      exec: {
        action: "echo",
        args: {
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
    }
  },
  assert: """
    (all
      (eq state "Ok")
      (eq value.2.1.item "1-c")
    )
  """
}


stage.s1.step.s2: {
  let: {
    content: "$ref:step.sp1.value"
  },
  exec: {
    action: "log",
    args: {
      log: "$ref:content"
    }
  }
}