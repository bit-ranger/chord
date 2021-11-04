version: "0.0.1"


stage.s1.step.step1: {
  let: {
    foo: "{{case.foo}}",
    bar: "{{case.bar}}",
    lua: """{{file "step1.lua"}}"""
  },
  exec: {
    action: "lua",
    args: {
      lua: "{{lua}}"
    }
  },
  assert: """
    (all
      (eq value.1.bar (num bar))
    )
  """
}

stage.s1.step.step2: {
  let: {
    foo: "{{case.foo}}",
    bar: "{{case.bar}}",
  },
  exec: {
    action: "lua",
    args: {
      lua: """
        r = os.time();
        t =  "CHORD-" .. tostring(r);
        print(t);
        return
        {
            {
                ['foo'] = foo
            }
        ,
            {
                ['bar'] = tonumber(bar)
            },
            {
                ['tag'] = t
            }
        }
      """
    }
  },
  assert: """
    (all
      (eq value.1.bar (num bar))
    )
  """
}