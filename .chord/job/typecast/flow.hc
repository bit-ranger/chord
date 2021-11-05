version: "0.0.1"

def: {
  a: 1
},


stage.s1.step.s1: {
  let: {
    arg1: "{{ref def.a}}"
    obj2: {
      x: "y"
    }
    arr2: [
      1, 2, 3
    ]
  },
  exec: {
    action: "echo",
    args: {
      echo: {
        ref: "{{ref arg1}}",
        num1: "{{num 456}}",
        bool1: "{{bool true}}",
        obj1: """{{obj "{\"x\":\"y\"}" }}""",
        obj2: """{{obj  obj2}}""",
        arr1: """{{arr "[1,2,3]" }}""",
        arr2: """{{arr arr2}}"""
      }
    }
  },
  assert: """
              (all
                  (eq value.ref arg1)
                  (eq value.num1 456)
                  (eq value.bool1 true)
                  (eq value.obj1.x "y")
                  (eq value.obj2.x "y")
                  (eq value.arr1.0 1)
                  (eq value.arr2.0 1)
                  (eq value.obj1 value.obj2)
                  (eq value.arr1 value.arr2)
              )
          """
}