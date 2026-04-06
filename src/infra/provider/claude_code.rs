use crate::app::ports::ProviderPort;
use crate::domain::asset::{AssetKind, ScannedPackage};
use crate::domain::identity::AssetIdentity;
use crate::domain::scope::Scope;
use crate::infra::provider::common::copy_dir;
use anyhow::Result;
use std::path::PathBuf;

pub struct ClaudeCodeProvider {
    workspace_root: PathBuf,
}

impl ClaudeCodeProvider {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    fn provider_root(&self, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Global => dirs_next::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".claude"),
            Scope::Workspace => self.workspace_root.join(".claude"),
        }
    }

    fn asset_dir(&self, scope: &Scope, kind: &AssetKind, name: &str) -> PathBuf {
        let root = self.provider_root(scope);
        match kind {
            AssetKind::Skill => root.join("skills").join(name),
            AssetKind::Instruction => root.join("instructions").join(name),
        }
    }
}

impl ProviderPort for ClaudeCodeProvider {
    fn id(&self) -> &str {
        "claude-code"
    }

    fn name(&self) -> &str {
        "Claude Code"
    }

    fn install(&self, pkg: &ScannedPackage, scope: Scope) -> Result<()> {
        let dest = self.asset_dir(&scope, &pkg.kind, &pkg.identity.name);
        copy_dir(&pkg.path, &dest)
    }

    fn remove(&self, identity: &AssetIdentity, kind: &AssetKind, scope: Scope) -> Result<()> {
        let dest = self.asset_dir(&scope, kind, &identity.name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::asset::AssetKind;
    use std::path::Path;

    fn make_pkg(dir: &Path, name: &str, kind: AssetKind, marker: &str) -> ScannedPackage {
        let pkg_dir = dir.join(name);
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join(marker), format!("# {}", name)).unwrap();
        ScannedPackage {
            identity: AssetIdentity::new(name, None, "0000000000"),
            path: pkg_dir,
            vault_id: "workspace".to_string(),
            kind,
            is_remote: false,
            remote_meta: None,
        }
    }

    #[test]
    fn install_skill_copies_to_workspace_claude_skills() {
        let dir = tempfile::tempdir().unwrap();
        let src_dir = dir.path().join("source");
        std::fs::create_dir(&src_dir).unwrap();
        let pkg = make_pkg(&src_dir, "my-skill", AssetKind::Skill, "SKILL.md");
        let provider = ClaudeCodeProvider::new(dir.path().to_path_buf());
        provider.install(&pkg, Scope::Workspace).unwrap();
        assert!(dir.path().join(".claude/skills/my-skill/SKILL.md").exists());
    }

    #[test]
    fn install_instruction_copies_to_workspace_claude_instructions() {
        let dir = tempfile::tempdir().unwrap();
        let src_dir = dir.path().join("source");
        std::fs::create_dir(&src_dir).unwrap();
        let pkg = make_pkg(&src_dir, "my-inst", AssetKind::Instruction, "AGENTS.md");
        let provider = ClaudeCodeProvider::new(dir.path().to_path_buf());
        provider.install(&pkg, Scope::Workspace).unwrap();
        assert!(dir
            .path()
            .join(".claude/instructions/my-inst/AGENTS.md")
            .exists());
    }

    #[test]
    fn remove_skill_deletes_directory() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".claude/skills/my-skill");
        std::fs::create_dir_all(&dest).unwrap();
        std::fs::write(dest.join("SKILL.md"), "x").unwrap();
        let provider = ClaudeCodeProvider::new(dir.path().to_path_buf());
        let identity = AssetIdentity::new("my-skill", None, "0000000000");
        provider
            .remove(&identity, &AssetKind::Skill, Scope::Workspace)
            .unwrap();
        assert!(!dest.exists());
    }

    #[test]
    fn remove_nonexistent_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let provider = ClaudeCodeProvider::new(dir.path().to_path_buf());
        let identity = AssetIdentity::new("ghost", None, "0000000000");
        let result = provider.remove(&identity, &AssetKind::Skill, Scope::Workspace);
        assert!(result.is_ok());
    }
}
