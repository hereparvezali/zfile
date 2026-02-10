use chrono::{DateTime, Local};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: SystemTime,
    #[allow(unused)]
    pub is_hidden: bool,
}

impl FileInfo {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let metadata = fs::metadata(path)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let is_hidden = name.starts_with('.');

        Ok(FileInfo {
            path: path.to_path_buf(),
            name,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::now()),
            is_hidden,
        })
    }

    pub fn format_size(&self) -> String {
        if self.is_dir {
            return "Folder".to_string();
        }

        let size = self.size as f64;

        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }

    pub fn format_modified(&self) -> String {
        let datetime: DateTime<Local> = self.modified.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn icon_name(&self) -> &'static str {
        if self.is_dir {
            "folder-symbolic"
        } else {
            // Simple icon selection based on extension
            if let Some(ext) = self.path.extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "txt" | "md" | "rst" => "text-x-generic-symbolic",
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" => {
                        "image-x-generic-symbolic"
                    }
                    "mp3" | "wav" | "ogg" | "flac" | "m4a" => "audio-x-generic-symbolic",
                    "mp4" | "avi" | "mkv" | "mov" | "webm" => "video-x-generic-symbolic",
                    "pdf" => "application-pdf-symbolic",
                    "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => {
                        "package-x-generic-symbolic"
                    }
                    "rs" | "py" | "js" | "c" | "cpp" | "h" | "java" | "go" => {
                        "text-x-script-symbolic"
                    }
                    "html" | "htm" | "xml" | "css" => "text-html-symbolic",
                    _ => "text-x-generic-symbolic",
                }
            } else {
                "text-x-generic-symbolic"
            }
        }
    }
}
