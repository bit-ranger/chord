let conf = {
  version: "0.0.1",
  stage: {
    smoking: {
      step: {}
    }
  }
};

module.exports = () => conf;
let step = conf.stage.smoking.step;

step.step1 = {
  let: {
    url: "{{case.url}}"
  },
  exec: {
    download: {
      url: "{{url}}"
    }
  },
  assert: `
      (eq state "Ok")
`
}


step.step2 = {
  let: {
    url: "{{case.url}}"
  },
  exec: {
    download: {
      header: {
        abc: [
          "a",
          "b",
          "c"
        ]
      },
      url: "{{url}}"
    }
  }
}

step.setp3 = {
  let: {
    step1_size: "{{step.step1.value.size}}"
  },
  exec: {
    log: "{{step1_size}}"
  }
}