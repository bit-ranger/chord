let conf = {
  version: "0.0.1",
  prelude: {
    step: {}
  },
  stage: {
    smoking: {
      step: {}
    }
  }
};

module.exports = () => conf;
let prelude = conf.prelude;
let smoking = conf.stage.smoking;


prelude.step.p1 = {
  exec: {
    program: {
      cmd: [
        "python3",
        "--version"
      ]
    }
  }
}

prelude.step.p2 = {
  exec: {
    program: {
      detach: true,
      cmd: [
        "python3",
        "--version"
      ]
    }
  }
}

smoking.step.step1 = {
  exec: {
    program: {
      cmd: [
        "python3",
        "--version"
      ]
    }
  }
}

smoking.step.step2 = {
  exec: {
    program: {
      detach: true,
      cmd: [
        "python3",
        "--version"
      ]
    }
  }
}

smoking.step.s3 = {
  exec: {
    program: {
      cmd: [
        "python3",
        "--version"
      ]
    }
  }
}