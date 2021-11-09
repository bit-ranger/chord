version: "0.0.1"


stage.s1.step.step1: {
  let: {
    lon: "{{case.origin_lon}}",
    lat: "{{case.origin_lat}}"
  },
  exec: {
    echo: """
        update coord set x = '{{lon}}', y = '{{lat}}'
      """
  }
}