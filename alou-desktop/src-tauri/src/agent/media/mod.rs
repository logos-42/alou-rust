//! 媒体模块

pub mod storage;

pub use storage::{save_media, get_media_directory, delete_media, get_media_metadata, MediaFileMetadata};
