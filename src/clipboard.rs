use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Clipboard Image Support (modeled after gemini-cli's clipboardUtils) ─────
// Linux: wl-paste (Wayland) or xclip (X11)

/// Detect which clipboard tool is available on Linux.
fn detect_linux_clipboard_tool() -> Option<&'static str> {
    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let tool = match session.as_str() {
        "wayland" => "wl-paste",
        "x11" => "xclip",
        _ => return None,
    };

    // Check if the tool is installed
    if Command::new("which")
        .arg(tool)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some(tool)
    } else {
        None
    }
}

/// Check if the system clipboard contains an image.
pub fn clipboard_has_image() -> bool {
    if let Some(tool) = detect_linux_clipboard_tool() {
        match tool {
            "wl-paste" => {
                Command::new("wl-paste")
                    .arg("--list-types")
                    .output()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout).contains("image/")
                    })
                    .unwrap_or(false)
            }
            "xclip" => {
                Command::new("xclip")
                    .args(["-selection", "clipboard", "-t", "TARGETS", "-o"])
                    .output()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout).contains("image/")
                    })
                    .unwrap_or(false)
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Save clipboard image to a temporary file.
/// Returns the path to the saved image, or None if no image or error.
pub fn save_clipboard_image(project_dir: &Path) -> Result<Option<PathBuf>> {
    let images_dir = project_dir.join(".guv").join("clipboard-images");
    std::fs::create_dir_all(&images_dir).context("Failed to create clipboard images dir")?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let dest = images_dir.join(format!("clipboard-{}.png", timestamp));

    if let Some(tool) = detect_linux_clipboard_tool() {
        let success = match tool {
            "wl-paste" => {
                Command::new("wl-paste")
                    .args(["--no-newline", "--type", "image/png"])
                    .stdout(std::fs::File::create(&dest).context("Failed to create image file")?)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            }
            "xclip" => {
                Command::new("xclip")
                    .args(["-selection", "clipboard", "-t", "image/png", "-o"])
                    .stdout(std::fs::File::create(&dest).context("Failed to create image file")?)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            }
            _ => false,
        };

        if success && dest.exists() {
            let size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
            if size > 0 {
                return Ok(Some(dest));
            }
        }

        // Clean up empty file
        let _ = std::fs::remove_file(&dest);
    }

    Ok(None)
}

/// Clean up clipboard images older than 1 hour.
pub fn cleanup_old_clipboard_images(project_dir: &Path) {
    let images_dir = project_dir.join(".guv").join("clipboard-images");
    if !images_dir.exists() {
        return;
    }

    let one_hour_ago = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(3600))
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("clipboard-"))
                .unwrap_or(false)
            {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        if modified < one_hour_ago {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
}
