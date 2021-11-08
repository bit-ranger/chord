version: "0.0.1"

stage.s1.step.step1: {
  let: {
    url: "{{case.url}}"
  },
  exec: {
    download: {
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
},

stage.s1.step.setp3: {
  let: {
    step1_size: "{{step.step1.value.size}}"
  },
  exec: {
    log: {
      log: "{{step1_size}}"
    }
  }
}