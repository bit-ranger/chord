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
            lnum: 456,
            lbool: true,
            lobj: {
                x: "y"
            },
            larr: [
                1, 2, 3
            ],

            num1: `{{num "456"}}`,
            num2: "{{num 456}}",
            num3: "{{num lnum}}",
            num4: "{{json 456}}",
            num5: "{{json lnum}}",

            bool1: `{{bool "true"}}`,
            bool2: "{{bool true}}",
            bool3: "{{bool lbool}}",
            bool4: "{{json true}}",
            bool5: "{{json lbool}}",

            obj1: `{{obj "{\\"x\\":\\"y\\"}" }}`,
            obj2: `{{obj {"x":"y"} }}`,
            obj3: "{{obj lobj}}",
            obj4: `{{json {"x":"y"} }}`,
            obj5: "{{json lobj}}",

            arr1: `{{arr "[1,2,3]" }}`,
            arr2: "{{arr [1,2,3] }}",
            arr3: "{{arr larr}}",
            arr4: "{{json [1,2,3] }}",
            arr5: "{{json larr}}"
        }
    },

    ok: {
        assert: `
(all
(eq var.num1 456)
(eq var.num2 456)
(eq var.num3 456)
(eq var.num4 456)
(eq var.num5 456)

(eq var.bool1 true)
(eq var.bool2 true)
(eq var.bool3 true)
(eq var.bool4 true)
(eq var.bool5 true)

(eq var.obj1.x "y")
(eq var.obj2.x "y")
(eq var.obj3.x "y")
(eq var.obj4.x "y")
(eq var.obj5.x "y")

(eq var.arr1.0 1)
(eq var.arr2.0 1)
(eq var.arr3.0 1)
(eq var.arr4.0 1)
(eq var.arr5.0 1)
)
`
    }
}
