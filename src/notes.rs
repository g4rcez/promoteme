use anyhow::Result;
use std::fs;
use std::path::Path;

/// Collect notes from .md and .txt files in a directory
pub fn collect_notes(dir: &Path) -> Result<String> {
    if !dir.is_dir() {
        return Ok(String::new());
    }

    let mut content = String::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" && ext != "txt" {
            continue;
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        let file_content = fs::read_to_string(&path)?;

        content.push_str("\n\n---\n");
        content.push_str(&format!("Notes from {}:\n", filename));
        content.push_str(&file_content);
    }

    Ok(content)
}
