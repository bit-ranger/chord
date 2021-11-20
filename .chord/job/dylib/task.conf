version: "0.0.1"

stage.s1.step.step1: {
  exec: {
    dylib: "chord_action_dylib_example"
  },
  assert: """
    (all
      (eq value.run "1")
    )
  """
}