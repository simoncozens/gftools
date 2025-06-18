use std::path::{Path, PathBuf};

use scraper::Html;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use skrifa::{raw::TableProvider, string::StringId, FontRef, MetadataProvider};

use crate::{error::GftoolsError, parse_metadatapb, DesignerInfoProto, FamilyProto};

fn parse_html(filename: &PathBuf) -> Result<String, GftoolsError> {
    let s = std::fs::read_to_string(filename)?;
    let document = Html::parse_fragment(&s);
    Ok(document
        .tree
        .nodes()
        .filter_map(|node| match node.value() {
            scraper::Node::Text(text) => Some(text.trim()),
            _ => None,
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Family {
    pub name: String,
    pub version: String,
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
                GftoolsError::Misc(format!(
                    "Couldn't find family in JSON: {}",
                    data.to_string()
                ))
            })?;
        Self::from_googlefonts(name, url)
    }
    pub(crate) fn from_googlefonts(_name: &str, _url: &str) -> Result<Self, GftoolsError> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AxisFallback {
    pub name: String,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Axis {
    pub tag: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "min")]
    pub min_value: f32,
    #[serde(rename = "defaultValue")]
    pub default_value: f32,
    #[serde(rename = "max")]
    pub max_value: f32,
    pub precision: f32,
    #[serde(default)]
    pub fallback: Vec<AxisFallback>,
    #[serde(default)]
    pub fallback_only: bool,
    pub description: String,
}

impl Axis {
    fn from_path(_path: &Path) -> Result<Self, GftoolsError> {
        unimplemented!()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FamilyMeta {
    #[serde(rename = "family")]
    pub name: String,
    #[serde(default, rename = "displayName")]
    pub display_name: Option<String>,
    pub designers: Vec<Designer>,
    pub license: String,
    pub category: String,
    #[serde(rename = "coverage", deserialize_with = "get_keys")]
    pub subsets: Vec<String>,
    #[serde(deserialize_with = "deserialize_null_default", default)]
    pub stroke: String,
    pub classifications: Vec<String>,
    pub description: String,
    pub primary_script: Option<String>,
    #[serde(deserialize_with = "deserialize_null_default", default)]
    pub article: Vec<String>,
    pub minisite_url: Option<String>,
}

fn get_keys<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // When it comes from the server, this is a map. But when we serialize it, we want a list of keys.
    // We want to be able to deserialize it from a map or a list of strings.
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    if let Some(s) = value.as_object() {
        return Ok(s.keys().cloned().collect());
    }
    if let Some(arr) = value.as_array() {
        return Ok(arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect());
    }
    Err(de::Error::custom("Expected a map or an array of strings"))
}

impl FamilyMeta {
    fn from_path(path: &Path) -> Result<Self, GftoolsError> {
        let data = parse_metadatapb::<FamilyProto>(path)?;
        let stroke = data
            .stroke
            .as_ref()
            .map(|x| x.replace("_,", " ").to_uppercase())
            .or_else(|| data.category.first().cloned())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        let article = path.join("article").join("ARTICLE.en_us.html");
        let article = if article.exists() {
            vec![parse_html(&article)?]
        } else {
            vec![]
        };
        let description = path.join("DESCRIPTION.en_us.html");
        let description = if description.exists() {
            Some(parse_html(&description)?)
        } else {
            None
        };
        Ok(Self {
            name: data.name().to_string(),
            designers: data
                .designer
                .as_ref()
                .map(|x| {
                    x.split(", ")
                        .map(|s| Designer {
                            name: s.to_string(),
                            bio: "".to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            license: data.license().to_string(),
            display_name: data.display_name.map(|x| x.to_string()),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Designer {
    pub name: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub bio: String,
}

impl Designer {
    pub(crate) fn from_path(path: &Path) -> Result<Self, GftoolsError> {
        let data = parse_metadatapb::<DesignerInfoProto>(path)?;
        let bio = path.join("bio.html");
        let bio = bio.exists().then(|| parse_html(&bio)).transpose()?;
        Ok(Self {
            name: data.designer().to_string(),
            bio: bio.unwrap_or_default(),
        })
    }
    pub(crate) fn from_googlefonts_json(data: Value, _url: &str) -> Result<Self, GftoolsError> {
        Ok(Self {
            name: data["designer"]
                .as_str()
                .ok_or(GftoolsError::Misc(format!(
                    "Couldn't find designer in JSON: {}",
                    data
                )))?
                .to_string(),
            bio: data["bio"]
                .as_str()
                .ok_or(GftoolsError::Misc(format!(
                    "Couldn't find bio in JSON: {}",
                    data
                )))?
                .to_string(),
        })
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) enum PushItem {
    Family(Family),
    AxisFallback(AxisFallback),
    Axis(Axis),
    FamilyMeta(FamilyMeta),
    Designer(Designer),
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
