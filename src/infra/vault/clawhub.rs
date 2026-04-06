use crate::app::ports::{FeatureSetPort, VaultPort};
use crate::domain::asset::ScannedPackage;
use crate::infra::vault::local::LocalVaultAdapter;
use anyhow::Result;
use std::path::PathBuf;

pub struct ClawHubVaultAdapter {
    id: String,
}

impl ClawHubVaultAdapter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait::async_trait]
impl VaultPort for ClawHubVaultAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind_name(&self) -> &str {
        "clawhub"
    }

    fn list_packages(&self, feature: &dyn FeatureSetPort) -> Result<Vec<ScannedPackage>> {
        let cache_dir = crate::domain::paths::clawhub_cache_dir();
        if !cache_dir.exists() {
            return Ok(Vec::new());
        }
        let local = LocalVaultAdapter::new(&self.id, cache_dir);
        local.list_packages(feature)
    }
}

/// Check if the `clawhub` CLI is available on $PATH.
pub fn is_cli_available() -> bool {
    std::process::Command::new("which")
        .arg("clawhub")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Homebrew is available (macOS).
pub fn is_homebrew_available() -> bool {
    std::process::Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Install clawhub CLI via Homebrew.
pub fn install_cli_via_homebrew() -> Result<()> {
    let status = std::process::Command::new("brew")
        .args(["install", "clawhub"])
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to install clawhub via Homebrew");
    }
    Ok(())
}

/// Run `clawhub search <query>` and parse results into ScannedPackages.
/// Output format per line: `slug  Display Name  (score)`
pub fn cli_search(query: &str) -> Result<Vec<ScannedPackage>> {
    let output = std::process::Command::new("clawhub")
        .args(["search", query])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("clawhub search failed: {}", stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<ScannedPackage> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            // Format: "slug  Display Name  (score)"
            // Slug is the first token (no spaces), separated by 2+ spaces
            let slug = line.split_whitespace().next()?;
            Some(ScannedPackage {
                identity: crate::domain::identity::AssetIdentity::new(slug, None, "----------"),
                path: PathBuf::new(),
                vault_id: "clawhub".to_string(),
                kind: crate::domain::asset::AssetKind::Skill,
                is_remote: true,
            })
        })
        .collect();
    Ok(packages)
}

/// Run `clawhub install <slug>` with workdir set to agk's clawhub cache.
pub fn cli_install(slug: &str) -> Result<()> {
    let cache_dir = crate::domain::paths::clawhub_cache_dir();
    std::fs::create_dir_all(&cache_dir)?;
    let status = std::process::Command::new("clawhub")
        .args(["install", slug, "--workdir", &cache_dir.to_string_lossy()])
        .status()?;
    if !status.success() {
        anyhow::bail!("clawhub install '{}' failed", slug);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::feature::skill::SkillFeatureSet;

    #[test]
    fn clawhub_vault_id() {
        let adapter = ClawHubVaultAdapter::new("clawhub");
        assert_eq!(adapter.id(), "clawhub");
    }

    #[test]
    fn clawhub_vault_kind_name() {
        let adapter = ClawHubVaultAdapter::new("clawhub");
        assert_eq!(adapter.kind_name(), "clawhub");
    }

    #[test]
    fn list_packages_empty_when_no_cache_dir() {
        let adapter = ClawHubVaultAdapter::new("clawhub");
        let pkgs = adapter.list_packages(&SkillFeatureSet).unwrap();
        assert!(pkgs.is_empty());
    }

    #[test]
    fn list_packages_finds_cached_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("skills").join("my-clawhub-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "# My Skill").unwrap();

        let local = LocalVaultAdapter::new("clawhub", dir.path().to_path_buf());
        let pkgs = local.list_packages(&SkillFeatureSet).unwrap();
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].identity.name, "my-clawhub-skill");
        assert_eq!(pkgs[0].vault_id, "clawhub");
    }
}
