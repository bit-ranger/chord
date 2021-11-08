version: "0.0.1"


stage.s1.step.step1: {
  let: {
    content: "{{case.content}}"
  },
  exec: {
    dubbo: {
      method: "com.bitranger.dubbo.provider.service.EchoService#echo(java.lang.String)",
      args: [
        "{{content}}"
      ]
    }
  },
  assert: """
    (all
      (eq value.code "0")
      (eq value.data content)
    )
  """
}