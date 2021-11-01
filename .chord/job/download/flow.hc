version: "0.0.1",
stage: {
  stage1: {
    step: {

    }
  }
}


stage.s1.step.step1: {
  let: {
    url: "{{case.url}}"
  },
  exec: {
    action: "download",
    args: {
      url: "{{url}}"
    }
  },
  assert: """
    (eq state "Ok")
  """
},

stage.s1.step.step2: {
  let: {
    url: "{{case.url}}"
  },
  exec: {
    action: "download",
    args: {
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
},

stage.s1.step.setp3: {
  let: {
    step1_size: "{{step.step1.value.size}}"
  },
  exec: {
    action: "log",
    args: {
      log: "{{step1_size}}"
    }
  }
}