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
        lat: "{{case.origin_lat}}",
        lon_lat: "{{lon}},{{lat}}"
    },

    log: "dml >>> update some_table set column1 = '{{lon_lat}}' where id = 1'"

}
