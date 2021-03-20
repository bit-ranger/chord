use common::case::{CaseResult, CaseState};
use common::error::Error;
use common::task::{TaskResult};
use common::point::PointState;
use crate::model::Error as CurrError;

pub async fn report<W: std::io::Write>(task_result: &TaskResult, writer: W) -> Result<(), Error> {
    match report0(task_result, writer).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.common())
    }
}

async fn report0<W: std::io::Write>(task_result: &TaskResult, writer: W) -> Result<(), CurrError> {
    let mut rwr = csv::WriterBuilder::new().from_writer(writer);
    let empty = &vec![];
    let cr_vec = match task_result {
        Ok(tr) => tr.result(),
        Err(_) => empty
    };

    if cr_vec.len() == 0 {
        return Ok(());
    }
    let head_vec = to_head_vec(cr_vec);

    let _ = rwr.write_record(&head_vec);
    for sv in cr_vec.iter().map(|(_, cr)| to_value_vec(cr, head_vec.len())){
        rwr.write_record(&sv)?
    }
    rwr.flush()?;
    return Ok(());
}


fn to_value_vec(cr: &CaseResult, head_len: usize) -> Vec<String> {

    let mut vec = vec![];
    match cr {
        Ok(ca) => {
            match ca.state() {
                CaseState::Ok => {
                    vec.push(String::from("O"));
                    vec.push(String::from(""));
                    vec.push(ca.start().format("%T").to_string());
                    vec.push(ca.end().format("%T").to_string());
                },
                CaseState::PointError(e) => {
                    vec.push(String::from("E"));
                    vec.push(String::from(format!("{}", e)));
                    vec.push(ca.start().format("%T").to_string());
                    vec.push(ca.end().format("%T").to_string());
                },
                CaseState::PointFailure => {
                    vec.push(String::from("F"));
                    vec.push(String::from(""));
                    vec.push(ca.start().format("%T").to_string());
                    vec.push(ca.end().format("%T").to_string());
                }
            }
        }
        Err(e) => {
            vec.push(String::from("E"));
            vec.push(String::from(format!("{}", e)));
            vec.push(String::from(""));
            vec.push(String::from(""));
        }
    };

    let empty = &vec![];
    let pr_vec = match cr {
        Ok(cr) => cr.result(),
        Err(_) =>  empty
    };

    if !pr_vec.is_empty() {
        let p_vec: Vec<String> = pr_vec.iter()
            .flat_map(|(_, pr)| match pr {
                Ok(pa) => {
                    match pa.state() {
                        PointState::Ok => {
                            vec![String::from("O"),
                                 pa.start().format("%T").to_string(),
                                 pa.end().format("%T").to_string(),
                            ]
                        },
                        PointState::Error(e) => {
                            vec![String::from("E"),
                                 String::from(""),
                                 String::from(""),
                                 format!("{}", e)]
                        },
                        PointState::Failure => {
                            vec![String::from("F"),
                                 pa.start().format("%T").to_string(),
                                 pa.end().format("%T").to_string(),
                                 format!("{}", pa.result())]
                        },
                    }
                },
                Err(e) => vec![String::from("E"),
                               String::from(""),
                               String::from(""),
                               format!("{}", e)]
            })
            .collect();
        vec.extend(p_vec);
    }

    if vec.len() < head_len -3 {
        for _i in 0..head_len -3 - vec.len() {
            vec.push(String::from(""));
        }
    }

    vec
}

fn to_head_vec(cr_vec: &Vec<(usize, CaseResult)>) -> Vec<String> {
    let mut vec:Vec<String> = vec![];
    vec.push(String::from("case_state"));
    vec.push(String::from("case_info"));
    vec.push(String::from("case_start"));
    vec.push(String::from("case_end"));

    let (_, max_len_cr) = cr_vec.iter().max_by(
        |(_, x), (_, y)| {
            let x = match x {
                Ok(pv) => pv.result().len(),
                Err(_) => 0
            };
            let y = match y {
                Ok(pv) => pv.result().len(),
                Err(_) => 0
            };
            x.cmp(&y)
        })
    .unwrap();

    let empty = &vec![];
    let pr_vec =  match max_len_cr {
        Ok(cr) => cr.result(),
        Err(_) => empty
    };

    let ph_vec: Vec<String> = pr_vec.iter()
        .map(|(pid, _)| pid)
        .flat_map(|pid| vec![format!("{}_state", pid), format!("{}_start", pid), format!("{}_end",pid)])
        .collect();
    vec.extend(ph_vec);
    vec.push(String::from("last_point_info"));
    vec
}

