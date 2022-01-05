let conf = {
    version: "0.0.1",
    stage: {
        smoking: {
            step: {}
        }
    }
};

module.exports = () => conf;
let step = conf.stage.smoking.step;

step.step1 = {
    log: "{{(num_sub 2.1 1.1)}}"
}

step.step2 = {

    nop: {},
    assert: `
      (all
        (eq (num_add 1 1) 2)
        (eq (num_add 1.1 1) 2.1)
        (eq (num_add 1.5 2.5) 4.0)
        (eq (num_sub 2.1 1.1) 1.0)
        (eq (num_mul 2.1 2) 4.2)
        (eq (num_div 2.2 2) 1.1)
    )
   `
}