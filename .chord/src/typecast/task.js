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
        lnum: 456,
        lbool: true,
        lobj: {
            x: "y"
        },
        larr: [
            1, 2, 3
        ]
    },

    echo: {
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
    },
    assert: `
(all
(eq value.num1 456)
(eq value.num2 456)
(eq value.num3 456)
(eq value.num4 456)
(eq value.num5 456)

(eq value.bool1 true)
(eq value.bool2 true)
(eq value.bool3 true)
(eq value.bool4 true)
(eq value.bool5 true)

(eq value.obj1.x "y")
(eq value.obj2.x "y")
(eq value.obj3.x "y")
(eq value.obj4.x "y")
(eq value.obj5.x "y")

(eq value.arr1.0 1)
(eq value.arr2.0 1)
(eq value.arr3.0 1)
(eq value.arr4.0 1)
(eq value.arr5.0 1)
)
`
}
