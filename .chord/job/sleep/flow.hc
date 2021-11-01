version: "0.0.1"

stage.s1.step.example: {
  let: {
    duration: "{{case.seconds}}"
  },
  exec: {
    action: "sleep",
    args: {
      duration: "{{duration}}"
    }
  },
  assert: """
    (all
      (eq 1 1)
      (eq 2 2)
      (eq 3 3)
    )
  """
}