version: "0.0.1",
def: {
  database: {
    url: "mysql://root:example@127.0.0.1:3306/mysql?useUnicode=true&characterEncoding=utf8&useSSL=false&serverTimezone=Asia/Shanghai"
  }
},
stage.s1.step.select: {
  let: {
    url: "{{def.database.url}}",
    user: "{{case.user}}"
  },
  exec: {
    action: "database",
    args: {
      url: "{{url}}",
      sql: "SELECT * FROM user WHERE user = '{{user}}'"
    }
  },
  assert: "(eq value.records.0.User user)"
}