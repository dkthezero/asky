use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct AssetIdentity {
    pub name: String,
    pub version: Option<String>,
    pub sha10: String,
}

impl AssetIdentity {
    pub fn new(name: impl Into<String>, version: Option<String>, sha10: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version,
            sha10: sha10.into(),
        }
    }
}

impl fmt::Display for AssetIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = self.version.as_deref().unwrap_or("--");
        write!(f, "[{}:{}:{}]", self.name, version, self.sha10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_with_version() {
        let id = AssetIdentity::new("web-tool", Some("1.2.0".to_string()), "a13c9ef042");
        assert_eq!(id.to_string(), "[web-tool:1.2.0:a13c9ef042]");
    }

    #[test]
    fn display_without_version() {
        let id = AssetIdentity::new("local-script", None, "9ac00ff113");
        assert_eq!(id.to_string(), "[local-script:--:9ac00ff113]");
    }

    #[test]
    fn name_accessor() {
        let id = AssetIdentity::new("my-skill", None, "0000000000");
        assert_eq!(id.name, "my-skill");
    }
}
