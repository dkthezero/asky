use crate::domain::identity::AssetIdentity;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum AssetKind {
    Skill,
    Instruction,
}

#[derive(Debug, Clone)]
pub struct ScannedPackage {
    pub identity: AssetIdentity,
    pub path: PathBuf,
    pub vault_id: String,
    pub kind: AssetKind,
}

/// Display-only struct for the Vaults tab.
#[derive(Debug, Clone)]
pub struct VaultEntry {
    pub id: String,
    pub kind: String,
    pub enabled: bool,
    pub installed_skills: usize,
    pub available_skills: usize,
    pub installed_instructions: usize,
    pub available_instructions: usize,
}

impl VaultEntry {
    pub fn counts_label(&self) -> String {
        format!(
            "{}/{}s  {}/{}i",
            self.installed_skills,
            self.available_skills,
            self.installed_instructions,
            self.available_instructions,
        )
    }
}

/// Display-only struct for the Providers tab.
#[derive(Debug, Clone)]
pub struct ProviderEntry {
    pub id: String,
    pub name: String,
    pub active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_kind_clone() {
        let k = AssetKind::Skill;
        assert_eq!(k.clone(), AssetKind::Skill);
    }

    #[test]
    fn asset_kind_eq() {
        assert_ne!(AssetKind::Skill, AssetKind::Instruction);
    }

    #[test]
    fn vault_entry_display_counts() {
        let entry = VaultEntry {
            id: "community".to_string(),
            kind: "github".to_string(),
            enabled: true,
            installed_skills: 30,
            available_skills: 48,
            installed_instructions: 8,
            available_instructions: 12,
        };
        assert_eq!(entry.id, "community");
        assert_eq!(entry.counts_label(), "30/48s  8/12i");
    }

    #[test]
    fn provider_entry_active_marker() {
        let entry = ProviderEntry {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            active: true,
        };
        assert!(entry.active);
    }

    #[test]
    fn scanned_package_name_via_identity() {
        let pkg = ScannedPackage {
            identity: AssetIdentity::new("my-skill", None, "abc1234567"),
            path: PathBuf::from("/skills/my-skill"),
            vault_id: "workspace".to_string(),
            kind: AssetKind::Skill,
        };
        assert_eq!(pkg.identity.name, "my-skill");
        assert_eq!(pkg.vault_id, "workspace");
    }
}
