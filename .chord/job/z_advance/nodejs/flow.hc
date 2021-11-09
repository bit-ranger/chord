version: "0.0.1"


def {
  prefix: """{{fs_path "nodejs-example"}}"""
}

stage.s1.step.s1: {
  let {
    prefix: "{{def.prefix}}"
  },
  exec: {
    program: {
      cmd: [
        "npm",
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
    prefix: "{{def.prefix}}"
  },
  exec: {
    program: {
      cmd: [
        "npm",
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