mod error;
mod push;
mod utils;

mod protos {
    #![allow(clippy::all, clippy::unwrap_used)]
    include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}
use std::path::Path;

#[allow(unused_imports)]
pub(crate) use designers::DesignerInfoProto;
use error::GftoolsError;
pub(crate) use fonts_public::FamilyProto;
use protos::{designers, fonts_public};

fn parse_metadatapb<T>(path: &Path) -> Result<T, GftoolsError>
where
    T: protobuf::MessageFull,
{
    let meta_file = path.join("METADATA.pb");
    let meta_contents = std::fs::read(meta_file)?;
    let contents = std::str::from_utf8(&meta_contents)
        .map_err(|_| GftoolsError::Misc("METADATA.pb is not valid UTF-8".to_string()))?;
    let data = protobuf::text_format::parse_from_str::<T>(contents)
        .map_err(GftoolsError::ProtobufParse)?;
    Ok(data)
}
