use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use skrifa::{raw::TableProvider, string::StringId, FontRef, MetadataProvider};

use crate::{error::GftoolsError, utils::parse_html, FamilyProto};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Family {
    name: String,
    version: String,
}

fn font_version(f: &FontRef) -> String {
    if let Some(version) = f
        .localized_strings(StringId::VERSION_STRING)
        .english_or_first()
        .map(|name| name.chars().collect())
    {
        version
    } else {
        f.head()
            .map(|head| head.font_revision().to_string())
            .unwrap_or_else(|_| "0.0.0".to_string())
    }
}

impl Family {
    pub(crate) fn from_fontref(f: FontRef) -> Self {
        let family: String = f
            .localized_strings(StringId::FAMILY_NAME)
            .english_or_first()
            .map(|name| name.chars().collect())
            .unwrap_or("Unknown family".to_string());
        let version: String = font_version(&f);
        Family {
            name: family,
            version,
        }
    }
    pub(crate) fn from_filepath(f: &PathBuf) -> Result<Self, GftoolsError> {
        let contents = std::fs::read(f)?;
        let font = skrifa::FontRef::new(&contents)?;
        Ok(Self::from_fontref(font))
    }
    pub(crate) fn from_googlefonts_json(data: Value, url: &str) -> Result<Self, GftoolsError> {
        let name = data
            .as_object()
            .and_then(|m| m.get("family"))
            .and_then(|f| f.as_str())
            .ok_or_else(|| {
                GftoolsError::JsonParse(format!("Couldn't find family in JSON: {}", data))
            })?;
        Self::from_googlefonts(name, url)
    }
    pub(crate) fn from_googlefonts(name: &str, url: &str) -> Result<Self, GftoolsError> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AxisFallback {
    name: String,
    value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Axis {
    tag: String,
    display_name: String,
    min_value: f32,
    default_value: f32,
    max_value: f32,
    precision: f32,
    #[serde(default)]
    fallback: Vec<AxisFallback>,
    #[serde(default)]
    fallback_only: bool,
    description: String,
}

impl Axis {
    fn from_path(path: &Path) -> Result<Self, GftoolsError> {
        unimplemented!()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FamilyMeta {
    name: String,
    designer: Vec<String>,
    license: String,
    category: String,
    subsets: Vec<String>,
    stroke: String,
    classifications: Vec<String>,
    description: String,
    primary_script: Option<String>,
    article: Option<String>,
    minisite_url: Option<String>,
}

impl FamilyMeta {
    fn from_path(path: &Path) -> Result<Self, GftoolsError> {
        let meta_path = path.join("METADATA.pb");
        let meta_file = std::fs::read(meta_path)?;
        let contents = std::str::from_utf8(&meta_file)
            .map_err(|_| GftoolsError::Misc("METADATA.pb is not valid UTF-8".to_string()))?;
        let data = protobuf::text_format::parse_from_str::<FamilyProto>(contents)
            .map_err(GftoolsError::ProtobufParse)?;
        let mut stroke = data.stroke().replace("_,", " ").to_uppercase();
        if stroke.is_empty() {
            stroke = data
                .category
                .first()
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string());
        }

        let article = path.join("article").join("ARTICLE.en_us.html");
        let article = if article.exists() {
            Some(parse_html(&article)?)
        } else {
            None
        };
        let description = path.join("DESCRIPTION.en_us.html");
        let description = if description.exists() {
            Some(parse_html(&description)?)
        } else {
            None
        };
        Ok(Self {
            name: data.name().to_string(),
            designer: data.designer().split(", ").map(|s| s.to_string()).collect(),
            license: data.license().to_string(),
            category: data.category.first().cloned().unwrap_or_default(),
            subsets: data.subsets,
            stroke,
            classifications: data.classifications,
            description: description.unwrap_or_default(),
            primary_script: data.primary_script,
            article,
            minisite_url: data.minisite_url,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Designer {
    name: String,
    bio: String,
}

enum PushItem {
    Family(Family),
    AxisFallback { name: String, value: f32 },
    Axis(Axis),
    FamilyMeta(FamilyMeta),
    Designer(Designer),
}
