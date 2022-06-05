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
            bar: "{{case.bar}}",
        },
    },
    value: {
        // language=Lua
        lua: `
            let = action("let")
            lv = let:run({
                "c", "b", "a"
            })
            assert(lv[1] == "c")
            
            count = action("count", {
                init = 1,
                incr = 1
            });
            assert(count:run() == 1)
            assert(count:run() == 2)
            assert(count:run() == 3)

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
                }
            ,
                {
                    ['tag'] = t
                }
            }
        `,
    },

    state: {
        // language=Lua
        lua: `
            assert(tostring(value[2].bar) == tostring(var.bar), "fail")
        `
    }
}

step.step2 = {
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
        // language=Lua
        lua: `
            assert(tostring(value[2].bar) == tostring(var.bar), "fail")
        `
    }
}


