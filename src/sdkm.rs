pub mod url {
    use std::error::Error;

    pub(crate) fn serialize<S>(url: &url::Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(url.as_str())
    }
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<url::Url, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let url_string: String = serde::Deserialize::deserialize(deserializer)?;
        url::Url::parse(&url_string).map_err(|e| serde::de::Error::custom(e.description()))
    }
}
