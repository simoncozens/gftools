mod error;
mod push;
mod utils;

mod protos {
    #![allow(clippy::all, clippy::unwrap_used)]
    include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}
#[allow(unused_imports)]
pub(crate) use designers::DesignerInfoProto;
pub(crate) use fonts_public::FamilyProto;
use protos::{designers, fonts_public};

// pub(crate) fn family_proto(t: &Testable) -> Result<FamilyProto, CheckError> {
//     let mdpb = std::str::from_utf8(&t.contents)
//         .map_err(|_| CheckError::Error("METADATA.pb is not valid UTF-8".to_string()))?;
//     protobuf::text_format::parse_from_str::<FamilyProto>(mdpb)
//         .map_err(|e| CheckError::Error(format!("Error parsing METADATA.pb: {}", e)))
// }
