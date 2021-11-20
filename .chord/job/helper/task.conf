version: "0.0.1",

stage.s1.step.s1: {
  exec: {
    log: "{{(num_sub 2.1 1.1)}}"
  }
},

stage.s1.step.s2: {
  exec: {
    nop: {}
  },
  assert: """
  (all
    (eq (num_add 1 1) 2)
    (eq (num_add 1.1 1) 2.1)
    (eq (num_add 1.5 2.5) 4.0)
    (eq (num_sub 2.1 1.1) 1.0)
    (eq (num_mul 2.1 2) 4.2)
    (eq (num_div 2.2 2) 1.1)
  )
  """
}