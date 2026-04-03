pub(crate) mod instruction;
pub(crate) mod skill;
pub(crate) mod stub;

pub fn extract_frontmatter_version(content: &str) -> Option<String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("---")?;
    let frontmatter = &rest[..end];
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("version:") {
            let version = value.trim().trim_matches('"').trim_matches('\'');
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }
    None
}
