version: "0.0.1"

def: {
  a: 1
},


stage.s1.step.s1: {
  let: {
    arg1: {
      "$ref": "def.a"
    }
  },
  exec: {
    action: "echo",
    args: {
      echo: {
        ref: {
          "$ref": "arg1"
        },
        str: {
          "$str": "abc"
        },
        num1: {
          "$num": 456
        },
        num2: {
          "$num": "456"
        },
        bool1: {
          "$bool": true
        },
        bool2: {
          "$bool": "true"
        },
        obj1: {
          "$obj": {
            x: "y"
          }
        },
        obj2: {
          "$obj": """
                      {
                        "x": "y"
                      }
                    """
        },
        arr1: {
          "$arr": [
            1,
            2,
            3
          ]
        },
        arr2: {
          "$arr": "[1,2,3]"
        }
      }
    }
  },
  assert: """
              (all
                  (eq value.ref arg1)
                  (eq value.str "abc")
                  (eq value.num1 456)
                  (eq value.num2 456)
                  (eq value.bool1 true)
                  (eq value.bool2 true)
                  (eq value.obj1.x "y")
                  (eq value.obj2.x "y")
                  (eq value.arr1.0 1)
                  (eq value.arr2.0 1)
              )
          """
}