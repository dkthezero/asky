use crate::app::ports::FeatureSetPort;
use crate::domain::asset::AssetKind;
use std::path::{Path, PathBuf};

pub struct StubFeatureSet {
    kind: &'static str,
    display: &'static str,
    root: &'static str,
}

impl StubFeatureSet {
    pub fn new(kind: &'static str, display: &'static str, root: &'static str) -> Self {
        Self {
            kind,
            display,
            root,
        }
    }
}

impl FeatureSetPort for StubFeatureSet {
    fn kind_name(&self) -> &str {
        self.kind
    }
    fn display_name(&self) -> &str {
        self.display
    }
    fn scan_root(&self) -> &str {
        self.root
    }
    fn asset_kind(&self) -> AssetKind {
        AssetKind::Instruction
    }
    fn is_package(&self, _: &Path) -> bool {
        false
    }
    fn hash_files(&self, _: &Path) -> Vec<PathBuf> {
        vec![]
    }
    fn is_stub(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ports::FeatureSetPort;

    #[test]
    fn stub_is_stub() {
        let s = StubFeatureSet::new("instruction", "Instructions", "instructions");
        assert!(s.is_stub());
    }

    #[test]
    fn stub_display_name() {
        let s = StubFeatureSet::new("provider", "Providers", "");
        assert_eq!(s.display_name(), "Providers");
    }

    #[test]
    fn stub_is_package_always_false() {
        let s = StubFeatureSet::new("vault", "Vaults", "");
        assert!(!s.is_package(std::path::Path::new("/any/path")));
    }
}
