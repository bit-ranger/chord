version: "0.0.1",
stage.s1.step.step1: {
  exec: {
    action: "count",
    args: {
      init: 10,
      incr: 2
    }
  },
  assert: """
        (all
            (gt value 9)
        )
        """
}