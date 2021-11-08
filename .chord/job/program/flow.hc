version: "0.0.1"

pre.step.p1 {
  exec: {
    action: "program",
    args: {
      program: "python3",
      args: [
        "--version"
      ]
    }
  }
}

pre.step.p2 {
  exec: {
    action: "program",
    args: {
      detach: true,
      program: "python3",
      args: [
        "--version"
      ]
    }
  }
}

stage.s1.step.s1: {
  exec: {
    action: "program",
    args: {
      program: "python3",
      args: [
        "--version"
      ]
    }
  }
}

stage.s1.step.s2: {
  exec: {
    action: "program",
    args: {
      detach: true,
      program: "python3",
      args: [
        "--version"
      ]
    }
  }
}

stage.s1.step.s3: {
  exec: {
    action: "program",
    args: {
      program: "python3",
      args: [
        "--version"
      ]
    }
  }
}