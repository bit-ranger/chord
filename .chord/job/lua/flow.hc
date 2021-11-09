version: "0.0.1"

def {
  lua: """{{fs_read "step1.lua"}}"""
}

stage.s1.step.step1: {
  let: {
    foo: "{{case.foo}}",
    bar: "{{case.bar}}",
    lua: "{{def.lua}}"
  },
  exec: {
    lua: "{{lua}}"
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
  },
  assert: """
    (all
      (eq value.1.bar (num bar))
    )
  """
}