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
    let: {
        foo: "{{case.foo}}",
        bar: "{{case.bar}}"
    },
    exec: {
        lua: fs.readFileSync(path.join(__dirname, "step1.lua"), {
                encoding: "utf-8"
            }
        )
    },
    assert: `
    (all
      (eq value.1.bar (num bar))
    )
  `
}

step.step2 = {
    let: {
        foo: "{{case.foo}}",
        bar: "{{case.bar}}",
    },
    exec: {
        // language=Lua
        lua: `
            r = os.time();
            t = "CHORD-" .. tostring(r);
            print(t);
            return
            {
                {
                    ['foo'] = foo
                }
            ,
                {
                    ['bar'] = tonumber(bar)
                },
                {
                    ['tag'] = t
                }
            }
        `
    },
    assert: `
    (all
      (eq value.1.bar (num bar))
    )
  `
}


