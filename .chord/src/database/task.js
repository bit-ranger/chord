let url = "mysql://root:example@10.10.139.79:3306/mysql?useUnicode=true&characterEncoding=utf8&useSSL=false&serverTimezone=Asia/Shanghai"

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
    let: {
        url: url,
        db: "{{case.db}}"
    },

    database: {
        url: "{{url}}",
        sql: "SELECT * FROM db where Db='{{db}}'"
    },
    assert: "(eq value.records.0.Db db)"
}