version: "0.0.1",
stage.s1.step.docker1: {
  spec: {
    timeout: 10
  },
  exec: {
    docker: {
      image: "ubuntu:20.04",
      value_to_json: true,
      cmd: [
        "echo",
        """{    "size": 100,    "from": 0,    "sort": {        "elapse": {            "order": "desc"        }    },    "query": {        "bool": {            "must": [                {                    "term": {                        "layer": "case"                    }                }            ]        }    }}"""
      ]
    }
  },
  assert: "(eq value.size 100)"
}