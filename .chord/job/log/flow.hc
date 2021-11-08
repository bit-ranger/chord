version: "0.0.1"

stage.s1.step.s1: {
  let: {
    lon: "{{case.origin_lon}}",
    lat: "{{case.origin_lat}}",
    lon_lat: "{{lon}},{{lat}}"
  },
  exec: {
    log: {
      log: "dml >>> update some_table set column1 = '{{lon_lat}}' where id = 1'"
    }
  }
}