use common::case::{CaseState, CaseAssess};
use common::error::Error;
use common::point::PointState;
use crate::model::PortError;
use csv::Writer;
use std::path::Path;
use common::flow::Flow;
use std::fs::File;
use common::task::{TaskAssess, TaskState};
use common::perr;

pub async fn report<W: std::io::Write>(writer: &mut Writer<W>,
                                       task_assess: &dyn TaskAssess,
                                       flow: &Flow,
) -> Result<(), Error> {
    match report0(writer, task_assess, flow).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.common())
    }
}

pub async fn from_writer<W: std::io::Write>(writer: W) -> Writer<W> {
    csv::WriterBuilder::new().from_writer(writer)
}

pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Writer<File>, Error> {
    csv::WriterBuilder::new().from_path(path).map_err(|e|perr!("csv", e.to_string()))
}

pub async fn prepare<W: std::io::Write>(writer: &mut Writer<W>, flow: &Flow) -> Result<(), Error> {
    writer.write_record(create_head(flow)).map_err(|e|perr!("csv", e.to_string()))
}

fn create_head(flow: &Flow) -> Vec<String> {
    let pt_id_vec: Vec<String> = flow.case_point_id_vec().unwrap_or(vec!());
    let mut vec: Vec<String> = vec![];
    vec.push(String::from("case_state"));
    vec.push(String::from("case_info"));
    vec.push(String::from("case_start"));
    vec.push(String::from("case_end"));

    let ph_vec: Vec<String> = pt_id_vec.iter()
        .flat_map(|pid| vec![format!("{}_state", pid), format!("{}_start", pid), format!("{}_end", pid)])
        .collect();
    vec.extend(ph_vec);
    vec.push(String::from("last_point_info"));
    vec
}

pub async fn write_record<W: std::io::Write>(writer: &mut Writer<W>, record: &Vec<String>) -> Result<(), Error> {
    match write_record0(writer, record).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.common())
    }
}

async fn write_record0<W: std::io::Write>(writer: &mut Writer<W>, record: &Vec<String>) -> Result<(), PortError> {
    Ok(writer.write_record(record)?)
}

async fn report0<W: std::io::Write>(writer: &mut Writer<W>, task_assess: &dyn TaskAssess, flow: &Flow) -> Result<(), PortError> {
    let empty = &vec![];
    let ca_vec = match task_assess.state() {
        TaskState::Ok(ca_vec) => ca_vec,
        TaskState::Fail(ca_vec) => ca_vec,
        TaskState::Err(_) => empty
    };

    if ca_vec.len() == 0 {
        return Ok(());
    }

    let head = create_head(flow);
    for sv in ca_vec.iter().map(|ca| to_value_vec(ca.as_ref(), head.len())) {
        writer.write_record(&sv)?
    }
    writer.flush()?;
    return Ok(());
}


fn to_value_vec(ca: &dyn CaseAssess, head_len: usize) -> Vec<String> {
    let mut vec = vec![];

    match ca.state() {
        CaseState::Ok(_) => {
            vec.push(String::from("O"));
            vec.push(String::from(""));
        }
        CaseState::Err(e) => {
            vec.push(String::from("E"));
            vec.push(String::from(format!("{}", e)));
        }
        CaseState::Fail(_) => {
            vec.push(String::from("F"));
            vec.push(String::from(""));
        }
    }
    vec.push(ca.start().format("%T").to_string());
    vec.push(ca.end().format("%T").to_string());

    let empty = &vec![];
    let pa_vec = match ca.state() {
        CaseState::Ok(pa_vec) => pa_vec,
        CaseState::Fail(pa_vec) => pa_vec,
        _ => empty
    };

    if !pa_vec.is_empty() {
        let p_vec: Vec<String> = pa_vec.iter()
            .flat_map(|pa| match pa.state() {
                PointState::Ok(_) => {
                    vec![
                        String::from("O"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string(),
                    ]
                }
                PointState::Err(_) => {
                    vec![
                        String::from("E"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string(),
                    ]
                }
                PointState::Fail(_) => {
                    vec![
                        String::from("F"),
                        pa.start().format("%T").to_string(),
                        pa.end().format("%T").to_string()
                    ]
                }
            })
            .collect();
        vec.extend(p_vec);
    }

    if vec.len() < head_len - 1 {
        for _i in 0..head_len - 1 - vec.len() {
            vec.push(String::from(""));
        }
    }

    if pa_vec.is_empty() {
        vec.push(String::from(""));
    } else {
        match pa_vec.last().unwrap().state(){
            PointState::Fail(json) => {
                vec.push(json.to_string());
            },
            PointState::Err(e) => {
                vec.push(e.to_string());
            }
            _ => {
                vec.push(String::from(""));
            }
        }
    }

    vec
}



