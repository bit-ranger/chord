version: "0.0.1"
let: {
  redis: {
    url: "redis://:@127.0.0.1:6379/0"
  }
}


stage.s1.step.set_redis: {
  let: {
    url: "{{let.redis.url}}",
    arg0: "{{case.arg0}}"
  },
  exec: {
    action: "redis",
    args: {
      url: "{{url}}",
      cmd: "SET",
      args: [
        "CHORD:TEST:0123456789",
        "{{arg0}}"
      ]
    }
  }
},
gstage.s1.step.et_redis: {
  let: {
    url: "{{let.redis.url}}",
    arg0: "{{case.arg0}}"
  },
  exec: {
    action: "redis",
    args: {
      url: "{{url}}",
      cmd: "GET",
      args: [
        "CHORD:TEST:0123456789"
      ]
    }
  },
  assert: "(eq value arg0)"
}