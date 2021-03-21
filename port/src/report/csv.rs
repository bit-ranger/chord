use common::case::{CaseResult, CaseState};
use common::error::Error;
use common::task::{TaskResult};
use common::point::PointState;
use crate::model::Error as CurrError;
use csv::Writer;
use std::path::Path;
use common::flow::Flow;
use std::fs::File;

pub async fn report<W: std::io::Write>(writer: &mut Writer<W>,
                                       task_result: &TaskResult,
    flow: &Flow
) -> Result<(), Error> {
    match report0(writer, task_result, flow).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.common())
    }
}

pub async fn from_writer<W: std::io::Write>(writer: W) -> Writer<W>{
    csv::WriterBuilder::new().from_writer(writer)
}

pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Writer<File>, Error>{
    csv::WriterBuilder::new().from_path(path).map_err(|e| Error::new("csv", e.to_string().as_str()))
}

pub async fn prepare<W: std::io::Write>(writer: &mut Writer<W>, flow: &Flow) -> Result<(),Error>{
    writer.write_record(create_head(flow)).map_err(|e|Error::new("csv", e.to_string().as_str()))
}

fn create_head(flow: &Flow) -> Vec<String>{
    let point_id_vec:Vec<String> =  flow.point_id_vec();
    let mut vec:Vec<String> = vec![];
    vec.push(String::from("case_state"));
    vec.push(String::from("case_info"));
    vec.push(String::from("case_start"));
    vec.push(String::from("case_end"));

    let ph_vec: Vec<String> = point_id_vec.iter()
        .flat_map(|pid| vec![format!("{}_state", pid), format!("{}_start", pid), format!("{}_end",pid)])
        .collect();
    vec.extend(ph_vec);
    vec.push(String::from("last_point_info"));
    vec
}

pub async fn write_record<W: std::io::Write>(writer: &mut Writer<W>, record: &Vec<String>) -> Result<(), Error> {
    match write_record0(writer, record).await{
        Ok(()) => Ok(()),
        Err(e) => Err(e.common())
    }
}

async fn write_record0<W: std::io::Write>(writer: &mut Writer<W>, record: &Vec<String>) -> Result<(), CurrError> {
    Ok(writer.write_record(record)?)
}

async fn report0<W: std::io::Write>(writer: &mut Writer<W>, task_result: &TaskResult, flow: &Flow) -> Result<(), CurrError> {


    let empty = &vec![];
    let cr_vec = match task_result {
        Ok(tr) => tr.result(),
        Err(_) => empty
    };

    if cr_vec.len() == 0 {
        return Ok(());
    }

    let head = create_head(flow);
    for sv in cr_vec.iter().map(|(_, cr)| to_value_vec(cr, head.len())){
        writer.write_record(&sv)?
    }
    writer.flush()?;
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

    let mut last_point_info = String::from("");
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
                            last_point_info = format!("{}", e);
                            vec![String::from("E"),
                                 String::from(""),
                                 String::from("")]
                        },
                        PointState::Failure => {
                            last_point_info = format!("{}", pa.result());
                            vec![String::from("F"),
                                 pa.start().format("%T").to_string(),
                                 pa.end().format("%T").to_string()]
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

    vec.push(last_point_info);
    vec
}



