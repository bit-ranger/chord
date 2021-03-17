use crate::model::context::{TaskResultInner, CaseResultInner };
use crate::model::error::Error;
use std::path::Path;

pub async fn export<P: AsRef<Path>>(task_result: &TaskResultInner, path: P) -> Result<(), Error> {
    let rwr = csv::Writer::from_path(path);
    let mut rwr = match rwr{
        Ok(w) => w,
        Err(_) => return Err(Error::new("000", "path error"))
    };

    let empty = &vec![];
    let cr_vec = match task_result {
        Ok(tr) => tr.result(),
        Err(_) => empty
    };

    if cr_vec.len() == 0 {
        return Ok(());
    }
    let name_vec = to_name_vec(cr_vec);

    let _ = rwr.write_record(&name_vec);
    cr_vec.iter()
        .map(|cr| case_result_to_value_vec(cr, name_vec.len()))
        .for_each(|sv| rwr.write_record(&sv).unwrap());

    rwr.flush()?;
    return Ok(());
}



fn case_result_to_value_vec(cr: &CaseResultInner, len: usize) -> Vec<String> {

    let empty = &vec![];
    let pr_vec = match cr {
        Ok(cr) => cr.result(),
        Err(e) => match e.get_attach(){
            Some(attach) => attach.result(),
            None => empty
        }
    };

    let mut vec: Vec<String> = pr_vec.iter()
        .map(|pr| match pr {
            Ok(_) => String::from("O"),
            Err(_) => String::from("X")
        })
        .collect();

    if vec.len() < len-3 {
        for _i in 0..len-3 - vec.len() {
            vec.push(String::from(""));
        }
    }

    match cr {
        Ok(_) => {
            vec.push(String::from("O"));
            vec.push(String::from(""));
            vec.push(String::from(""));
        },
        Err(e) => {
            vec.push(String::from("X"));
            vec.push(format!("{}", e));
            let  pr = pr_vec.last().unwrap();
            vec.push(match pr {
                Ok(pr) => format!("{}", pr.result()),
                Err(e) => format!("{}", e)
            })
        }
    };
    vec
}

fn to_name_vec(cr_vec: &Vec<(usize, CaseResultInner)>) -> Vec<String> {

    let max_len_cr = cr_vec.iter().max_by(
        |x, y| {
            let x = x.len();
            let y = y.len();
            x.cmp(&y)
        })
    .unwrap();

    let empty = &vec![];
    let pr_vec =  match max_len_cr {
        Ok(pr_vec) => pr_vec.result(),
        Err(e) => empty
    };

    let mut vec: Vec<String> = pr_vec.iter()
        .map(|pr| match pr {
            Ok(pr) => pr.id(),
            Err(e) => e.get_attach().unwrap().id()
        })
        .map(|id| String::from(id))
        .collect();
    vec.push(String::from("caseResult"));
    vec.push(String::from("caseInfo"));
    vec.push(String::from("lastPointInfo"));
    vec
}

