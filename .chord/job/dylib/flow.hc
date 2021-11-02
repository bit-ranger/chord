version: "0.0.1"

stage.s1.step.step1: {
  exec: {
    action: "dylib",
    args: {
      lib: "chord_action_dylib_example"
    }
  },
  assert: """
    (all
      (eq value.run "1")
    )
  """
}