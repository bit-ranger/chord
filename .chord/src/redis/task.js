let url = "redis://:@127.0.0.1:6379/0";

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

step.set_redis = {
    var: {
        let: {
            arg0: "{{case.arg0}}"
        }
    },

    value: {
        redis: {
            url: url,
            cmd: "SET",
            args: [
                "CHORD:TEST:0123456789",
                "{{var.arg0}}"
            ]
        }
    }
}

step.get_redis = {
    var: {
        let: {
            arg0: "{{case.arg0}}"
        }
    },

    value: {
        redis: {
            url: url,
            cmd: "GET",
            args: [
                "CHORD:TEST:0123456789"
            ]
        }
    },
    state: {
        assert: "(eq value var.arg0)"
    }
}
