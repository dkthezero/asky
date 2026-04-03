use crate::app::ports::FeatureSetPort;
use crate::domain::asset::AssetKind;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SkillFeatureSet;

impl FeatureSetPort for SkillFeatureSet {
    fn kind_name(&self) -> &str {
        "skill"
    }
    fn display_name(&self) -> &str {
        "Skills"
    }
    fn scan_root(&self) -> &str {
        "skills"
    }
    fn asset_kind(&self) -> AssetKind {
        AssetKind::Skill
    }

    fn is_package(&self, path: &Path) -> bool {
        path.join("SKILL.md").exists()
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
        let content = std::fs::read_to_string(path.join("SKILL.md")).ok()?;
        super::extract_frontmatter_version(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ports::FeatureSetPort;

    #[test]
    fn skill_feature_set_kind_name() {
        assert_eq!(SkillFeatureSet.kind_name(), "skill");
    }

    #[test]
    fn skill_feature_set_detects_package() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "# My Skill").unwrap();
        assert!(SkillFeatureSet.is_package(&skill_dir));
    }

    #[test]
    fn skill_feature_set_rejects_non_package() {
        let dir = tempfile::tempdir().unwrap();
        let other_dir = dir.path().join("not-a-skill");
        std::fs::create_dir(&other_dir).unwrap();
        std::fs::write(other_dir.join("README.md"), "nothing").unwrap();
        assert!(!SkillFeatureSet.is_package(&other_dir));
    }

    #[test]
    fn skill_feature_set_hash_files_includes_all_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "skill").unwrap();
        std::fs::write(dir.path().join("notes.md"), "notes").unwrap();
        let files = SkillFeatureSet.hash_files(dir.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn skill_feature_set_is_not_stub() {
        assert!(!SkillFeatureSet.is_stub());
    }

    #[test]
    fn extract_version_from_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\nversion: 2.1.0\n---\n# My Skill\n",
        )
        .unwrap();
        let version = SkillFeatureSet.extract_version(&skill_dir);
        assert_eq!(version, Some("2.1.0".to_string()));
    }

    #[test]
    fn extract_version_none_when_no_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "# My Skill\n").unwrap();
        let version = SkillFeatureSet.extract_version(&skill_dir);
        assert!(version.is_none());
    }
}
