version: "0.0.1",
stage.s1.step.docker1: {
  spec: {
    timeout: 10
  },
  exec: {
    action: "docker",
    args: {
      image: "ubuntu:20.04",
      cmd: [
        "echo",
        """{    "size": 100,    "from": 0,    "sort": {        "elapse": {            "order": "desc"        }    },    "query": {        "bool": {            "must": [                {                    "term": {                        "layer": "case"                    }                }            ]        }    }}"""
      ]
    }
  },
  assert: "(eq value.size 100)"
}