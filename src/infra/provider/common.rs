use anyhow::Result;
use std::path::Path;

pub fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        let _ = std::fs::remove_dir_all(dest);
    }
    std::fs::create_dir_all(dest)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            continue;
        }
        let rel = entry.path().strip_prefix(src)?;
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Remove a directory and prune empty parent directories up to `max_parent_levels`
/// levels above the removed directory. A `max_parent_levels` of 1 removes the
/// asset directory and then prunes its immediate parent (e.g. `skills/` or
/// `instructions/`) if that becomes empty, but never goes further up the tree.
pub fn remove_dir_and_prune_empty_parents(dir: &Path, max_parent_levels: usize) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(dir)?;
    let mut current = dir;
    for _ in 0..max_parent_levels {
        if let Some(parent) = current.parent() {
            if parent.exists() && is_dir_empty(parent)? {
                std::fs::remove_dir(parent)?;
            }
            current = parent;
        } else {
            break;
        }
    }
    Ok(())
}

fn is_dir_empty(path: &Path) -> Result<bool> {
    Ok(std::fs::read_dir(path)?.next().is_none())
}
