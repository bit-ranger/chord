mod controller;
use controller::job::exec as job__exec;
use tide::{Request, Response};
use chord_common::value::Json;
use chord_common::error::Error;
use tide::prelude::*;
use tide::http::StatusCode;

#[derive(Serialize, Deserialize)]
struct ErrorBody{
    code: String,
    message: String
}

fn common_error_json(e: &Error) -> Json {
    json!(ErrorBody{
                code: e.code().into(),
                message: e.message().into()
            })
}

// fn validator_error_json(e: &ValidationErrors) -> Json {
//     json!(ErrorBody{
//                 code: "400".into(),
//                 message: "illegal argument".into()
//             })
// }


#[macro_export]
macro_rules! json_handler {
    ($func:ident) => {{
        |mut req: Request<()>| async move {
            let rb =  req.body_json().await?;
            let rst = $func(rb).await;
            match rst{
                Ok(r) => Ok(Response::builder(StatusCode::Ok)
                    .body(json!(r))),
                Err(e) => Ok(Response::builder(StatusCode::InternalServerError)
                    .body(common_error_json(&e)))
            }
        }
    }}
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/job/exec").post(
        json_handler!(job__exec)
    );

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}


