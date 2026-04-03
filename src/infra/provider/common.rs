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
