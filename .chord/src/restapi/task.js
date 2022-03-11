let conf = {
    version: "0.0.1",
    pre: {
        step: {}
    },
    stage: {
        smoking: {
            round: 1,
            duration: 30,
            concurrency: 1,
            loader: {
                strategy: "fix_size_repeat_last_page"
            },
            step: {}
        }
    }
}

module.exports = () => conf;
let url_root = "http://127.0.0.1:9200";

let step = conf.stage.smoking.step;

step.del_idx = {


    value: {
        restapi: {
            url: url_root + "/article",
            method: "DELETE"
        }
    }
}

step.crt_inx = {

    value: {
        restapi: {
            url: url_root + "/article",
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
    ok: {
        assert:
            `
    (all
      (eq value.status 200)
      (eq value.body.acknowledged true)
    )
  `
    }
}


step.insert = {
    var: {
        let: {
            author: "{{case.author}}",
            title: "{{case.title}}",
            desc: "{{case.desc}}"
        }
    },

    value: {
        restapi: {
            url: url_root + "/article/_doc/1",
            method: "PUT",
            body: {
                "author": "{{var.author}}",
                "title": "{{var.title}}",
                "desc": "{{var.desc}}"
            }
        }
    },

    ok: {
        assert:
            `
    (all
      (eq value.status 201)
      (eq value.body.result "created")
    )
  `
    }
}


step.wait = {
    sleep: 9
}


step.search = {
    var: {
        let: {
            match: "{{case.match}}",
            term: "{{case.term}}"
        }
    },

    value: {
        restapi: {
            url: url_root + "/article/_search",
            method: "GET",
            body: {
                "size": 10,
                "from": 0,
                "query": {
                    "bool": {
                        "must": [
                            {
                                "match": {
                                    "desc": "{{var.match}}"
                                }
                            },
                            {
                                "term": {
                                    "author": "{{var.term}}"
                                }
                            }
                        ]
                    }
                }
            }
        }
    },

    ok: {
        assert:
            `
    (all
      (eq value.status 200)
      (eq value.body.hits.total.value 1)
    )
  `
    }
}
