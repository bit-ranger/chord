#[derive(Debug)]
pub struct ErrorStruct{
    code: String,
    message: String
}

impl ErrorStruct{

    pub fn new(code: &str, message: &str) -> ErrorStruct{
        ErrorStruct{
            code: String::from(code),
            message: String::from(message)
        }
    }

}
