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
    value: {
        redis: {
            __init__: {
                url: url,
            },
            cmd: "SET",
            args: [
                "CHORD:TEST:0123456789",
                "{{case.arg0}}"
            ]
        }
    }
}

step.get_redis = {
    value: {
        redis: {
            __init__: {
                url: url,
            },
            cmd: "GET",
            args: [
                "CHORD:TEST:0123456789"
            ]
        }
    },
    state: {
        assert: "(eq value case.arg0)"
    }
}
