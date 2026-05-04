use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Telemetry configuration and data stored in ~/.config/agk/analytics.toml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    #[serde(default)]
    pub settings: AnalyticsSettings,
    #[serde(default)]
    pub skills: HashMap<String, SkillAnalytics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSettings {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub last_scan: Option<String>,
}

impl Default for AnalyticsSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            last_scan: None,
        }
    }
}

fn default_enabled() -> bool {
    false
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillAnalytics {
    pub total_invocations: u64,
    pub last_used: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
}

impl AnalyticsConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn increment_invocation(
        &mut self,
        skill_name: &str,
        provider_id: &str,
    ) {
        let entry = self.skills.entry(skill_name.to_string()).or_default();
        entry.total_invocations += 1;
        entry.last_used = Some(chrono::Utc::now().to_rfc3339());
        if !entry.providers.contains(&provider_id.to_string()) {
            entry.providers.push(provider_id.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_disabled() {
        let config = AnalyticsConfig::default();
        assert!(!config.settings.enabled);
    }

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("analytics.toml");

        let mut config = AnalyticsConfig::default();
        config.settings.enabled = true;
        config.increment_invocation("web-browser", "claude-code");
        config.save(&path).unwrap();

        let loaded = AnalyticsConfig::load(&path).unwrap();
        assert!(loaded.settings.enabled);
        let skill = loaded.skills.get("web-browser").unwrap();
        assert_eq!(skill.total_invocations, 1);
        assert!(skill.providers.contains(&"claude-code".to_string()));
    }
}
