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
        content: "{{case.content}}"
    },
    exec: {
        dubbo: {
            method: "com.bitranger.dubbo.provider.service.EchoService#echo(java.lang.String)",
            args: [
                "{{content}}"
            ]
        }
    },
    assert: `
      (all
        (eq value.code "0")
        (eq value.data content)
      )
    `
}

