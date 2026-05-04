pub(crate) mod instruction;
pub(crate) mod skill;
pub(crate) mod stub;

/// Parsed YAML frontmatter from SKILL.md or AGENTS.md.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frontmatter {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub requires: Vec<String>,
    pub requires_optional: Vec<String>,
}

pub fn extract_frontmatter(content: &str) -> Option<Frontmatter> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("---")?;
    let frontmatter_text = &rest[..end];

    let mut fm = Frontmatter::default();
    let mut current_array: Option<&mut Vec<String>> = None;
    for line in frontmatter_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("version:") {
            let v = trimmed["version:".len()..].trim().trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                fm.version = Some(v.to_string());
            }
            current_array = None;
        } else if trimmed.starts_with("name:") {
            let v = trimmed["name:".len()..].trim().trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                fm.name = Some(v.to_string());
            }
            current_array = None;
        } else if trimmed.starts_with("description:") {
            let v = trimmed["description:".len()..].trim().trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                fm.description = Some(v.to_string());
            }
            current_array = None;
        } else if trimmed.starts_with("requires_optional:") {
            current_array = Some(&mut fm.requires_optional);
        } else if trimmed.starts_with("requires:") {
            current_array = Some(&mut fm.requires);
        } else if trimmed.starts_with("-") {
            let item = trimmed[1..].trim().trim_matches('"').trim_matches('\'');
            if !item.is_empty() {
                if let Some(arr) = current_array.as_deref_mut() {
                    arr.push(item.to_string());
                }
            }
        }
    }
    Some(fm)
}

pub fn extract_frontmatter_version(content: &str) -> Option<String> {
    extract_frontmatter(content).and_then(|fm| fm.version)
}
