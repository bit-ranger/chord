version: "0.0.1"

stage.s1.step.s1: {
  exec: {
    action: "program",
    args: {
      program: "java",
      args: [
        "-version"
      ],
      value_to_json: false
    }
  }
}

stage.s1.step.s2: {
  exec: {
    action: "program",
    args: {
      lifetime: "case",
      program: "python",
      args: [
        "--version"
      ],
      value_to_json: false
    }
  }
}

stage.s1.step.s3: {
  exec: {
    action: "program",
    args: {
      lifetime: "task",
      program: "python",
      args: [
        "--version"
      ],
      value_to_json: false
    }
  }
}