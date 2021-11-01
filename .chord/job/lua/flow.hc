version: "0.0.1"


stage.s1.step.step1: {
  let: {
    foo: "{{case.foo}}",
    bar: "{{case.bar}}"
  },
  exec: {
    action: "lua",
    args: {
      code: """
        r = os.time();
        t =  "CHORDV-" .. tostring(r);
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

stage.s1.step.step2: {
  let: {
    arr1: [
      "a",
      "b"
    ],
    arr2: {
      foo: "bar"
    }
  },
  exec: {
    action: "lua",
    args: {
      code: """
        table.insert(arr1, "c");
        table.insert(arr1, arr2);
        return arr1;
      """
    }
  },
  assert: """
    (all
      (eq value.1 "b")
      (eq value.2 "c")
      (eq value.3.foo "bar")
    )
  """
}