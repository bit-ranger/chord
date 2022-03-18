const path = require("path")
const fs = require("fs")

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
            bar: "{{case.bar}}"
        },
    },
    value: {
        lua: fs.readFileSync(path.join(__dirname, "step1.lua"), {
                encoding: "utf-8"
            }
        ),
    },
    state: {
        assert: `
        (all
        (eq value.1.bar (num var.bar))
        )
        `
    }
}

step.step2 = {
    var: {
        let: {
            foo: "{{case.foo}}",
            bar: "{{case.bar}}",
        },
    },
    value: {
        // language=Lua
        lua: `
            r = os.time();
            t = "CHORD-" .. tostring(r);
            print(t);
            return
            {
                {
                    ['foo'] = var.foo
                }
            ,
                {
                    ['bar'] = tonumber(var.bar)
                },
                {
                    ['tag'] = t
                }
            }
        `,
    },

    state: {
        assert: `
            (all
                (eq value.1.bar (num var.bar))
            )
        `
    }


}


