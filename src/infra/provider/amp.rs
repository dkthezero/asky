use crate::app::ports::ProviderPort;
use crate::domain::asset::{AssetKind, ScannedPackage};
use crate::domain::identity::AssetIdentity;
use crate::domain::scope::Scope;
use crate::infra::provider::common::copy_dir;
use anyhow::Result;
use std::path::PathBuf;

pub struct AmpProvider {
    workspace_root: PathBuf,
}

impl AmpProvider {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    fn provider_root(&self, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Global => dirs_next::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".amp"),
            Scope::Workspace => self.workspace_root.join(".amp"),
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
}

impl ProviderPort for AmpProvider {
    fn id(&self) -> &str {
        "amp"
    }

    fn name(&self) -> &str {
        "AMP Code"
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
