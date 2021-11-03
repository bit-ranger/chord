version: "0.0.1"

def: {
  arg: {
    arg1: "123",
    arg2: [
      "a",
      "b",
      "c"
    ]
  }
}


stage.s1.step.example: {
  let: {
    arg2: "$ref:def.arg.arg2"
   },
  exec: {
    action: "echo",
    args: {
      echo: [
        "hello",
        {
          hello: "world"
        },
        {
          "$ref:arg2"
        }
      ]
    }
  },
  assert: """
    (all
      (eq value.0 "hello")
      (eq value.1.hello "world")
      (eq value.2.1 "b")
    )
  """
}

stage.s1.step.example2: {
  let: {
    arg1: {
      "$ref:def.arg.arg1"
    },
    lon: "{{case.origin_lon}}",
    lat: "{{case.origin_lat}}"
  },
  exec: {
    action: "echo",
    args: {
      echo: "update bas set a = '{{lon}}' where b = '{{lat}}'"
    }
  },
  assert: """
    (all
      (str_start_with arg1 "12")
      (str_end_with arg1 "23")
      (str_contains arg1 "2")
      (eq
        (str_sub arg1 1) "23"
      )
      (eq
        (str_sub arg1 1 2)
        "2"
      )
    )
  """
}


stage.s2.step.example3: {
  let: {
    arg1: {
      "$ref:def.arg.arg1"
    },
    foo: "{{case.foo}}",
    bar: "{{case.bar}}"
  },
  exec: {
    action: "echo",
    args: {
      echo: "update bas set a = '{{foo}}' where b = '{{bar}}'"
    }
  }
  assert: """
    (all
      (str_start_with arg1 "12")
      (str_end_with arg1 "23")
      (str_contains arg1 "2")
      (eq (str_sub arg1 1) "23")
      (eq (str_sub arg1 1 2) "2")
  )
  """
}