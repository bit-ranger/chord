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
    let: {
        lon: "{{case.origin_lon}}",
    },

    log: "dml >>> update some_table set column1 = '{{lon}},{{case.origin_lat}}' where id = 1"

}
