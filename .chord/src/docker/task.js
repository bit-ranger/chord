let conf = {
    version: "0.0.1",
    stage: {
        smoking: {
            loader: {
                strategy: "fix_size_repeat_last_page"
            },
            step: {}
        }
    }
};

module.exports = () => conf;
let step = conf.stage.smoking.step;

step.step1 = {
    value: {
        docker: {
            image: "ubuntu:20.04",
            value_to_json: true,
            cmd: [
                "echo",
                `{    "size": 100,    "from": 0,    "sort": {        "elapse": {            "order": "desc"        }    },    "query": {        "bool": {            "must": [                {                    "term": {                        "layer": "case"                    }                }            ]        }    }}`
            ]
        },
    },
    ok: {
        assert: "(eq value.size 100)"
    }
}
