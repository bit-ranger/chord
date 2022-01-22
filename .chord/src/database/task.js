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
    let: {
        url: url,
        user: "{{case.user}}"
    },

    database: {
        url: "{{url}}",
        sql: "SELECT * FROM user WHERE user = '{{user}}'"
    },
    assert: "(eq value.records.0.User user)"
}