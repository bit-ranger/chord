version: "0.0.1"


stage.s1.step.s1: {
  exec: {
    action: "program",
    args: {
      program: "npm",
      args: [
        "--prefix",
        "/home/runner/work/chord/chord/zero/test/nodejs",
        "install"
      ],
      value_to_json: false
    }
  }
}

stage.s1.step.s2: {
  let: {
    case: "$ref:case"

  },
  exec: {
    action: "program",
    args: {
      program: "npm",
      args: [
        "--prefix",
        "/home/runner/work/chord/chord/zero/test/nodejs",
        "run",
        "test",
        "$ref:case"
      ],
      value_to_json: true
    }
  },
  assert: """
    (eq value.case_args.foo "bar")
  """
}