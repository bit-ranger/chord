let url = "mysql://root:example@127.0.0.1:3306/mysql?useUnicode=true&characterEncoding=utf8&useSSL=false&serverTimezone=Asia/Shanghai"

let conf = {
    version: "0.0.1",
    stage: {
        smoking: {
            step: {}
        }
    }
}
module.exports = () => conf;

let step = conf.stage.smoking.step;

step.step1 = {
    value: {
        database: {
            __init__: {
                url: url
            },
            sql: "SELECT * FROM db where Db='{{case.db}}'"
        },
    },

    ok: {
        assert: "(eq value.records.0.Db case.db)"
    }


}
