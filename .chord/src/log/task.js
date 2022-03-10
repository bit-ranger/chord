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
    var: {
        let: {
            lon: "{{case.origin_lon}}",
        }
    },

    value: {
        log: "dml >>> update some_table set column1 = '{{var.lon}},{{case.origin_lat}}' where id = 1"
    }
}
