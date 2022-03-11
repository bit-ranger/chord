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
            content: "{{case.content}}"
        }
    },

    value: {
        dubbo: {
            method: "com.bitranger.dubbo.provider.service.EchoService#echo(java.lang.String)",
            args: [
                "{{var.content}}"
            ]
        }
    },
    ok: {
        assert: `
      (all
        (eq value.code "0")
        (eq value.data var.content)
      )
    `
    }
}

