use base64::Engine as _;
use rocket::serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt::Display};

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Metadata(HashMap<String, String>);

#[derive(Debug, PartialEq)]
pub enum MetadataError {
    InvalidKey,
    DecodeError(String),
    InvalidMetadataFormat,
}

impl Error for MetadataError {}

impl Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Metadata {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_raw(&self, key: &str) -> Result<Vec<u8>, MetadataError> {
        let value = match self.0.get(key) {
            Some(v) => v,
            None => return Err(MetadataError::InvalidKey),
        };

        match base64::engine::general_purpose::STANDARD.decode(value) {
            Ok(decoded) => Ok(decoded),
            Err(e) => Err(MetadataError::DecodeError(e.to_string())),
        }
    }
}

impl TryFrom<&str> for Metadata {
    type Error = MetadataError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(MetadataError::InvalidMetadataFormat);
        }

        let mut metadata = Metadata::new();

        for pair in value.split(',') {
            let pair = pair.trim();

            if pair.is_empty() {
                continue;
            }

            let parts: Vec<&str> = pair.split(' ').map(|v| v.trim()).collect();

            if parts.is_empty() || parts.len() > 2 {
                return Err(MetadataError::InvalidMetadataFormat);
            }

            if parts[0].is_empty() {
                return Err(MetadataError::InvalidKey);
            }

            if let (Some(key), value) = (parts.get(0), parts.get(1)) {
                let value = match value {
                    Some(v) => v.to_string(),
                    None => String::default(),
                };

                metadata.0.insert(key.to_string(), value);
            }
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use super::*;

    const METADATA_STR: &str =
        "relativePath bnVsbA==, filename bXlfdmlkZW8ubXA0, filetype dmlkZW8vbXA0,is_confidential";

    #[test]
    fn valid_from_str() {
        let metadata = Metadata::try_from(METADATA_STR).unwrap();

        assert_eq!(metadata.0.len(), 4);
        assert_eq!(
            metadata.0.get("relativePath"),
            Some(&String::from("bnVsbA=="))
        );
        assert_eq!(
            metadata.0.get("filetype"),
            Some(&String::from("dmlkZW8vbXA0"))
        );
        assert_eq!(
            metadata.0.get("filename"),
            Some(&String::from("bXlfdmlkZW8ubXA0"))
        );
    }

    #[test]
    fn empty_from_str_error() {
        let metadata = Metadata::try_from("");

        assert!(metadata.is_err());
        assert_eq!(metadata.err(), Some(MetadataError::InvalidMetadataFormat));
    }

    #[test]
    fn invalid_format_from_str_error() {
        let metadata = Metadata::try_from("foobar, fas bars foo bar, ");

        assert!(metadata.is_err());
        assert_eq!(metadata.err(), Some(MetadataError::InvalidMetadataFormat));
    }

    #[test]
    fn get_raw_value_successfully() {
        let metadata = Metadata::try_from(METADATA_STR).unwrap();

        assert_eq!(metadata.get_raw("filetype"), Ok(b"video/mp4".to_vec()));
        assert_eq!(metadata.get_raw("filename"), Ok(b"my_video.mp4".to_vec()));
        assert_eq!(
            from_utf8(&metadata.get_raw("relativePath").unwrap()),
            Ok("null")
        );
    }

    #[test]
    fn get_raw_value_error() {
        let metadata = Metadata::try_from(METADATA_STR).unwrap();

        assert_eq!(
            metadata.get_raw("foo bar bars"),
            Err(MetadataError::InvalidKey)
        );
    }
}
