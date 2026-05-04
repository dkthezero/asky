use crate::app::ports::ProviderPort;
use crate::domain::asset::{AssetKind, ScannedPackage};
use crate::domain::identity::AssetIdentity;
use crate::domain::scope::Scope;
use crate::infra::provider::common::copy_dir;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct OpenCodeProvider {
    workspace_root: PathBuf,
}

impl OpenCodeProvider {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    fn provider_root(&self, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Global => dirs_next::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("opencode"),
            Scope::Workspace => self.workspace_root.join(".opencode"),
        }
    }

    fn asset_dir(&self, scope: &Scope, kind: &AssetKind, name: &str) -> PathBuf {
        let root = self.provider_root(scope);
        match kind {
            AssetKind::Skill => root.join("skills").join(name),
            AssetKind::Instruction => root.join("instructions").join(name),
        }
    }

    fn config_path(&self, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Global => dirs_next::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("opencode")
                .join("opencode.json"),
            Scope::Workspace => self.workspace_root.join("opencode.json"),
        }
    }

    fn load_config(&self, scope: &Scope) -> Result<OpenCodeConfig> {
        let path = self.config_path(scope);
        if !path.exists() {
            return Ok(OpenCodeConfig::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let cleaned = strip_jsonc_comments(&content);
        let config: OpenCodeConfig = serde_json::from_str(&cleaned)
            .with_context(|| format!("parsing JSON config at {}", path.display()))?;
        Ok(config)
    }

    fn save_config(&self, scope: &Scope, config: &OpenCodeConfig) -> Result<()> {
        let path = self.config_path(scope);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }
}

impl ProviderPort for OpenCodeProvider {
    fn id(&self) -> &str {
        "opencode"
    }

    fn name(&self) -> &str {
        "OpenCode"
    }

    fn install(&self, pkg: &ScannedPackage, scope: Scope) -> Result<()> {
        let dest = self.asset_dir(&scope, &pkg.kind, &pkg.identity.name);
        copy_dir(&pkg.path, &dest)?;

        // Merge skill reference into opencode.json
        let mut config = self.load_config(&scope)?;
        let skill_ref = SkillRef {
            name: pkg.identity.name.clone(),
            path: dest.to_string_lossy().into_owned(),
        };
        config.skills.retain(|s| s.name != pkg.identity.name);
        config.skills.push(skill_ref);
        self.save_config(&scope, &config)?;

        Ok(())
    }

    fn remove(&self, identity: &AssetIdentity, kind: &AssetKind, scope: Scope) -> Result<()> {
        let dest = self.asset_dir(&scope, kind, &identity.name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }

        // Remove skill reference from opencode.json
        let mut config = self.load_config(&scope)?;
        config.skills.retain(|s| s.name != identity.name);
        self.save_config(&scope, &config)?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Config types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct OpenCodeConfig {
    #[serde(default)]
    skills: Vec<SkillRef>,
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillRef {
    name: String,
    path: String,
}

// ---------------------------------------------------------------------------
// JSONC comment stripper (basic)
// ---------------------------------------------------------------------------

fn strip_jsonc_comments(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                result.push('\n');
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' {
                if let Some(&'/') = chars.peek() {
                    chars.next();
                    in_block_comment = false;
                }
            }
            continue;
        }

        if in_string {
            result.push(ch);
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            result.push(ch);
            continue;
        }

        if ch == '/' {
            match chars.peek() {
                Some(&'/') => {
                    chars.next();
                    in_line_comment = true;
                    continue;
                }
                Some(&'*') => {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        result.push(ch);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::asset::AssetKind;

    fn make_pkg(
        dir: &std::path::Path,
        name: &str,
        kind: AssetKind,
        marker: &str,
    ) -> ScannedPackage {
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
    fn install_skill_copies_to_workspace_opencode_skills() {
        let dir = tempfile::tempdir().unwrap();
        let src_dir = dir.path().join("source");
        std::fs::create_dir(&src_dir).unwrap();
        let pkg = make_pkg(&src_dir, "my-skill", AssetKind::Skill, "SKILL.md");
        let provider = OpenCodeProvider::new(dir.path().to_path_buf());
        provider.install(&pkg, Scope::Workspace).unwrap();
        assert!(dir
            .path()
            .join(".opencode/skills/my-skill/SKILL.md")
            .exists());
    }

    #[test]
    fn install_instruction_copies_to_workspace_opencode_instructions() {
        let dir = tempfile::tempdir().unwrap();
        let src_dir = dir.path().join("source");
        std::fs::create_dir(&src_dir).unwrap();
        let pkg = make_pkg(&src_dir, "my-inst", AssetKind::Instruction, "AGENTS.md");
        let provider = OpenCodeProvider::new(dir.path().to_path_buf());
        provider.install(&pkg, Scope::Workspace).unwrap();
        assert!(dir
            .path()
            .join(".opencode/instructions/my-inst/AGENTS.md")
            .exists());
    }

    #[test]
    fn install_updates_opencode_json() {
        let dir = tempfile::tempdir().unwrap();
        let src_dir = dir.path().join("source");
        std::fs::create_dir(&src_dir).unwrap();
        let pkg = make_pkg(&src_dir, "my-skill", AssetKind::Skill, "SKILL.md");
        let provider = OpenCodeProvider::new(dir.path().to_path_buf());
        provider.install(&pkg, Scope::Workspace).unwrap();

        let config_path = dir.path().join("opencode.json");
        assert!(config_path.exists());
        let content = std::fs::read_to_string(config_path).unwrap();
        assert!(content.contains("my-skill"));
        assert!(content.contains(".opencode/skills/my-skill"));
    }

    #[test]
    fn remove_skill_deletes_directory_and_updates_config() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join(".opencode/skills/my-skill");
        std::fs::create_dir_all(&dest).unwrap();
        std::fs::write(dest.join("SKILL.md"), "x").unwrap();

        // Pre-populate config
        let config_path = dir.path().join("opencode.json");
        std::fs::write(
            &config_path,
            r#"{"skills":[{"name":"my-skill","path":".opencode/skills/my-skill"}]}"#,
        )
        .unwrap();

        let provider = OpenCodeProvider::new(dir.path().to_path_buf());
        let identity = AssetIdentity::new("my-skill", None, "0000000000");
        provider
            .remove(&identity, &AssetKind::Skill, Scope::Workspace)
            .unwrap();
        assert!(!dest.exists());

        let content = std::fs::read_to_string(config_path).unwrap();
        assert!(!content.contains("my-skill"));
    }

    #[test]
    fn remove_nonexistent_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let provider = OpenCodeProvider::new(dir.path().to_path_buf());
        let identity = AssetIdentity::new("ghost", None, "0000000000");
        let result = provider.remove(&identity, &AssetKind::Skill, Scope::Workspace);
        assert!(result.is_ok());
    }

    #[test]
    fn strip_jsonc_line_comments() {
        let input = r#"{
            // This is a comment
            "key": "value"
        }"#;
        let cleaned = strip_jsonc_comments(input);
        assert!(!cleaned.contains("// This is a comment"));
        assert!(cleaned.contains("\"key\": \"value\""));
    }

    #[test]
    fn strip_jsonc_block_comments() {
        let input = r#"{
            /* This is a
               block comment */
            "key": "value"
        }"#;
        let cleaned = strip_jsonc_comments(input);
        assert!(!cleaned.contains("/* This is a"));
        assert!(cleaned.contains("\"key\": \"value\""));
    }

    #[test]
    fn merge_preserves_other_keys() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("opencode.json");
        std::fs::write(
            &config_path,
            r#"{"customKey": "customValue", "skills": []}"#,
        )
        .unwrap();

        let src_dir = dir.path().join("source");
        std::fs::create_dir(&src_dir).unwrap();
        let pkg = make_pkg(&src_dir, "my-skill", AssetKind::Skill, "SKILL.md");
        let provider = OpenCodeProvider::new(dir.path().to_path_buf());
        provider.install(&pkg, Scope::Workspace).unwrap();

        let content = std::fs::read_to_string(config_path).unwrap();
        assert!(content.contains("customKey"));
        assert!(content.contains("my-skill"));
    }
}
