version: "0.0.1"


let {
  prefix: """{{fs_path "nodejs-example"}}"""
}

stage.s1.step.s1: {
  let {
    prefix: "{{let.prefix}}"
  },
  exec: {
    action: "program",
    args: {
      program: "npm",
      args: [
        "--prefix",
        "{{prefix}}",
        "install"
      ],
      value_to_json: false
    }
  }
}

stage.s1.step.s2: {
  let: {
    case: "{{obj case}}",
    prefix: "{{let.prefix}}"
  },
  exec: {
    action: "program",
    args: {
      program: "npm",
      args: [
        "--prefix",
        "{{prefix}}",
        "run",
        "test",
        "{{obj case}}"
      ],
      value_to_json: true
    }
  },
  assert: """
    (eq value.case_args.foo "bar")
  """
}