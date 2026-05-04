use crate::app::ports::{McpProvider, ProviderPort};
use crate::domain::asset::{AssetKind, ScannedPackage};
use crate::domain::identity::AssetIdentity;
use crate::domain::mcp::McpServer;
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
            AssetKind::McpServer => PathBuf::new(),
        }
    }

    fn mcp_json_path(&self, scope: &Scope) -> PathBuf {
        self.provider_root(scope).join("mcp.json")
    }

    fn load_mcp_config(&self, scope: &Scope) -> Result<serde_json::Value> {
        let path = self.mcp_json_path(scope);
        if !path.exists() {
            return Ok(serde_json::json!({ "mcpServers": {} }));
        }
        let content = std::fs::read_to_string(&path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn save_mcp_config(&self, scope: &Scope, config: &serde_json::Value) -> Result<()> {
        let path = self.mcp_json_path(scope);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&path, content)?;
        Ok(())
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

    fn install_path_for(
        &self,
        identity: &AssetIdentity,
        kind: &AssetKind,
        scope: Scope,
    ) -> Option<PathBuf> {
        Some(self.asset_dir(&scope, kind, &identity.name))
    }
}

impl McpProvider for ClaudeCodeProvider {
    fn provider_id(&self) -> &str {
        "claude-code"
    }

    fn supports_mcp(&self) -> bool {
        true
    }

    fn mcp_config_path(&self, scope: Scope) -> Option<PathBuf> {
        Some(self.mcp_json_path(&scope))
    }

    fn write_mcp_server(&self, server: &McpServer, scope: Scope) -> Result<()> {
        let mut config = self.load_mcp_config(&scope)?;
        if config.get("mcpServers").is_none() {
            config["mcpServers"] = serde_json::json!({});
        }
        let mcp_servers = config["mcpServers"]
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!(".claude/mcp.json 'mcpServers' key is not an object"))?;

        let entry = serde_json::json!({
            "command": server.command,
            "args": server.args,
            "env": server.env,
        });
        mcp_servers.insert(server.name.clone(), entry);
        self.save_mcp_config(&scope, &config)
    }

    fn remove_mcp_server(&self, name: &str, scope: Scope) -> Result<()> {
        let mut config = self.load_mcp_config(&scope)?;
        if let Some(servers) = config
            .as_object_mut()
            .and_then(|obj| obj.get_mut("mcpServers"))
            .and_then(|v| v.as_object_mut())
        {
            servers.remove(name);
        }
        self.save_mcp_config(&scope, &config)
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
            requires: vec![],
            requires_optional: vec![],
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
