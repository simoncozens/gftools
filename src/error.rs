use thiserror::Error;

#[derive(Error, Debug)]
pub enum GftoolsError {
    #[error("problem parsing font")]
    FontParse(#[from] skrifa::raw::ReadError),
    #[error("problem parsing JSON")]
    JsonParse(String),
    #[error("problem parsing HTML")]
    HtmlParse(String),
    #[error("problem reading file")]
    FileRead(#[from] std::io::Error),
    #[error("miscellaneous error")]
    Misc(String),
    #[error("problem parsing protobuf")]
    ProtobufParse(#[from] protobuf::text_format::ParseError),
}
