version: "0.0.1"


stage.s1.step.s1: {
  exec: {
    iter_map: {
      arr: [
        "a",
        "b",
        "c"
      ],
      exec: {
        echo: {
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
    content: "{{arr step.s1.value}}"
  },
  exec: {
    log: {
      log: "{{arr content}}"
    }
  }
}