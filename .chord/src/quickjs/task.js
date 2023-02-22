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
            foo: "{{case.foo}}",
            bar: "{{case.bar}}",
        },
    },
    value: {
        // language=JavaScript
        quickjs: `async function a() {
            console.log('a called at %s ms', new Date().getTime());
            let res = await chord.action.Http.newClient(123);
            console.log('a got result %s at %s ms', res, new Date().getTime());
        };a();
        ;
        `,
    },
}


