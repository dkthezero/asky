use anyhow::{bail, Result};

pub fn validate_git_repo(repo: &str) -> Result<()> {
    // Regex matches `^[a-zA-Z0-9._-]+/[a-zA-Z0-9._-]+$`
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        bail!("Invalid git repository format: '{}'. Expected 'owner/repo'.", repo);
    }
    
    for part in parts {
        if part.is_empty() || part == ".." || !part.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-') {
            bail!("Invalid git repository component: '{}' in '{}'. Only alphanumeric, '.', '_', and '-' are allowed, and '..' is forbidden.", part, repo);
        }
    }
    Ok(())
}

pub fn validate_git_ref(r#ref: &str) -> Result<()> {
    if r#ref.starts_with('-') {
        bail!("Invalid git ref: '{}'. Cannot start with '-'.", r#ref);
    }
    if r#ref.chars().any(|c| c.is_control()) {
        bail!("Invalid git ref: '{}'. Cannot contain control characters.", r#ref);
    }
    Ok(())
}

pub fn validate_git_path(path: &str) -> Result<()> {
    if path.starts_with('-') {
        bail!("Invalid git subfolder path: '{}'. Cannot start with '-'.", path);
    }
    if path.contains("..") {
        bail!("Invalid git subfolder path: '{}'. Traversal with '..' is not allowed.", path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_git_repo() {
        assert!(validate_git_repo("owner/repo").is_ok());
        assert!(validate_git_repo("owner.name/repo-name_1").is_ok());
        assert!(validate_git_repo("owner/repo/extra").is_err());
        assert!(validate_git_repo("owner").is_err());
        assert!(validate_git_repo("owner/--repo").is_ok()); // This is fine for repo name if git allows it, but it doesn't start with -- as whole arg.
        // Actually the refactor-plan says: `repo` matches `^[a-zA-Z0-9._-]+/[a-zA-Z0-9._-]+$`
        assert!(validate_git_repo("owner/;rm -rf").is_err());
    }

    #[test]
    fn test_validate_git_ref() {
        assert!(validate_git_ref("main").is_ok());
        assert!(validate_git_ref("v1.0.0").is_ok());
        assert!(validate_git_ref("--upload-pack").is_err());
        assert!(validate_git_ref("feat/branch").is_ok());
    }

    #[test]
    fn test_validate_git_path() {
        assert!(validate_git_path("skills/").is_ok());
        assert!(validate_git_path("src/lib.rs").is_ok());
        assert!(validate_git_path("../external").is_err());
        assert!(validate_git_path("--config").is_err());
    }
}
