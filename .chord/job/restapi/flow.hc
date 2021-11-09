version: "0.0.1"
def: {
  es: {
    url: "http://127.0.0.1:9200"
  }
}
stage: {
  s1: {
    round: 1,
    duration: 30,
  }
}


stage.s1.step.del_idx: {
  let: {
    url: "{{def.es.url}}"
  },
  exec: {
    restapi: {
      url: "{{url}}/article",
      method: "DELETE"
    }
  }
}

stage.s1.step.crt_inx: {
  let: {
    url: "{{def.es.url}}"
  },
  exec: {
    restapi: {
      url: "{{url}}/article",
      method: "PUT",
      body: {
        "settings": {
          "index": {
            "analysis.analyzer.default.type": "ik_max_word"
          }
        },
        "mappings": {
          "properties": {
            "user": {
              "type": "text",
              "analyzer": "ik_max_word",
              "search_analyzer": "ik_max_word"
            },
            "title": {
              "type": "text",
              "analyzer": "ik_max_word",
              "search_analyzer": "ik_max_word"
            },
            "desc": {
              "type": "text",
              "analyzer": "ik_max_word",
              "search_analyzer": "ik_max_word"
            }
          }
        }
      }
    }
  },
  assert: """
    (all
      (eq value.status 200)
      (eq value.body.acknowledged true)
    )
  """
}


stage.s1.step.insert: {
  let: {
    url: "{{def.es.url}}",
    author: "{{case.author}}",
    title: "{{case.title}}",
    desc: "{{case.desc}}"
  },
  exec: {
    restapi: {
      url: "{{url}}/article/_doc/1",
      method: "PUT",
      body: {
        "author": "{{author}}",
        "title": "{{title}}",
        "desc": "{{desc}}"
      }
    }
  },
  assert: """
    (all
      (eq value.status 201)
      (eq value.body.result "created")
    )
  """
}


stage.s1.step.wait: {
  exec: {
    sleep: 9
  }
}


stage.s1.step.search: {
  let: {
    url: "{{def.es.url}}",
    match: "{{case.match}}",
    term: "{{case.term}}"
  },
  exec: {
    restapi: {
      url: "{{url}}/article/_search",
      method: "GET",
      body: {
        "size": 10,
        "from": 0,
        "query": {
          "bool": {
            "must": [
              {
                "match": {
                  "desc": "{{match}}"
                }
              },
              {
                "term": {
                  "author": "{{term}}"
                }
              }
            ]
          }
        }
      }
    }
  },
  assert: """
    (all
      (eq value.status 200)
      (eq value.body.hits.total.value 1)
    )
  """
}