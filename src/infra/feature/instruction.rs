use crate::app::ports::FeatureSetPort;
use crate::domain::asset::AssetKind;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct InstructionFeatureSet;

impl FeatureSetPort for InstructionFeatureSet {
    fn kind_name(&self) -> &str {
        "instruction"
    }
    fn display_name(&self) -> &str {
        "Instructions"
    }
    fn scan_root(&self) -> &str {
        "instructions"
    }
    fn asset_kind(&self) -> AssetKind {
        AssetKind::Instruction
    }

    fn is_package(&self, path: &Path) -> bool {
        path.join("AGENTS.md").exists()
    }

    fn hash_files(&self, path: &Path) -> Vec<PathBuf> {
        WalkDir::new(path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    fn extract_version(&self, path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path.join("AGENTS.md")).ok()?;
        super::extract_frontmatter_version(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ports::FeatureSetPort;

    #[test]
    fn instruction_feature_set_detects_agents_md() {
        let dir = tempfile::tempdir().unwrap();
        let pkg_dir = dir.path().join("my-instruction");
        std::fs::create_dir(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("AGENTS.md"), "# My Instruction").unwrap();
        assert!(InstructionFeatureSet.is_package(&pkg_dir));
    }

    #[test]
    fn instruction_feature_set_rejects_without_agents_md() {
        let dir = tempfile::tempdir().unwrap();
        let pkg_dir = dir.path().join("not-an-instruction");
        std::fs::create_dir(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("README.md"), "nope").unwrap();
        assert!(!InstructionFeatureSet.is_package(&pkg_dir));
    }

    #[test]
    fn instruction_feature_set_name_is_folder_name() {
        assert_eq!(InstructionFeatureSet.kind_name(), "instruction");
        assert_eq!(InstructionFeatureSet.display_name(), "Instructions");
    }

    #[test]
    fn instruction_feature_set_hash_files_includes_all_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "content").unwrap();
        std::fs::write(dir.path().join("notes.md"), "notes").unwrap();
        let files = InstructionFeatureSet.hash_files(dir.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn instruction_feature_set_is_not_stub() {
        assert!(!InstructionFeatureSet.is_stub());
    }

    #[test]
    fn instruction_asset_kind_is_instruction() {
        assert_eq!(InstructionFeatureSet.asset_kind(), AssetKind::Instruction);
    }

    #[test]
    fn extract_version_from_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let instruction_dir = dir.path().join("my-instruction");
        std::fs::create_dir_all(&instruction_dir).unwrap();
        std::fs::write(
            instruction_dir.join("AGENTS.md"),
            "---\nname: my-instruction\nversion: 3.0.0\n---\n# My Instruction\n",
        )
        .unwrap();
        let version = InstructionFeatureSet.extract_version(&instruction_dir);
        assert_eq!(version, Some("3.0.0".to_string()));
    }

    #[test]
    fn extract_version_none_when_no_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let instruction_dir = dir.path().join("my-instruction");
        std::fs::create_dir_all(&instruction_dir).unwrap();
        std::fs::write(instruction_dir.join("AGENTS.md"), "# My Instruction\n").unwrap();
        let version = InstructionFeatureSet.extract_version(&instruction_dir);
        assert!(version.is_none());
    }
}
