use ratatui::prelude::Color;
use crate::color::{ACCENT, CYAN, GREEN, ORANGE, PINK, PURPLE, TEXT_DIM, YELLOW};

#[derive(Debug, Clone, PartialEq)]
#[derive(Eq, Hash)]
pub enum ItemType {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Folder,
    Other,
}

impl ItemType {
    pub fn icon(&self) -> &str {
        match self {
            ItemType::Folder   => "󰉋 ",
            ItemType::Code     => "󰅩 ",
            ItemType::Document => "󰈙 ",
            ItemType::Image    => "󰈟 ",
            ItemType::Video    => "󰈫 ",
            ItemType::Audio    => "󰈣 ",
            ItemType::Archive  => "󰿺 ",
            ItemType::Other    => "󰈚 ",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ItemType::Folder => ACCENT,
            ItemType::Code => GREEN,
            ItemType::Document => CYAN,
            ItemType::Image => YELLOW,
            ItemType::Video => PURPLE,
            ItemType::Audio => PINK,
            ItemType::Archive => ORANGE,
            ItemType::Other => TEXT_DIM,
        }
    }

    pub fn from_extension(ext: &str, is_dir: bool) -> Self {
        if is_dir {
            return ItemType::Folder;
        }
        match ext.to_lowercase().as_str() {
            "pdf" | "doc" | "docx" | "txt" | "md" | "rtf" | "odt" => ItemType::Document,
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" => ItemType::Image,
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => ItemType::Video,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => ItemType::Audio,
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => ItemType::Archive,
            "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "go" | "rb" | "php"
            | "swift" | "kt" | "scala" | "html" | "css" | "json" | "yaml" | "toml" | "xml" | "sh" => ItemType::Code,
            _ => ItemType::Other,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ItemType::Folder => "Folders",
            ItemType::Code => "Code",
            ItemType::Document => "Docs",
            ItemType::Image => "Images",
            ItemType::Video => "Video",
            ItemType::Audio => "Audio",
            ItemType::Archive => "Archives",
            ItemType::Other => "Other",
        }
    }
}