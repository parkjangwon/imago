use crate::error::{ImagoError, Result};
use chrono::Local;
use colored::Colorize;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use viuer::{get_kitty_support, is_iterm_supported, print, Config, KittySupport};

/// Handles image saving and terminal display
pub struct ImageHandler {
    width: u32,
    height: Option<u32>,
    enable_preview: bool,
}

impl ImageHandler {
    /// Create a new image handler
    pub fn new(width: u32, height: Option<u32>, enable_preview: bool) -> Self {
        Self {
            width,
            height,
            enable_preview,
        }
    }

    /// Generate a filename with timestamp and random suffix
    pub fn generate_filename() -> String {
        let timestamp = Local::now().format("%Y%m%d%H%M");
        let random_str: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            .take(8)
            .map(char::from)
            .collect();
        format!("{}_{}.png", timestamp, random_str)
    }

    /// Resolve the output path
    pub fn resolve_output_path(&self, output: Option<&Path>) -> PathBuf {
        let filename = Self::generate_filename();

        match output {
            Some(path) => {
                if path.is_dir() || path.as_os_str().to_string_lossy().ends_with('/') {
                    path.join(filename)
                } else {
                    let path_str = path.as_os_str().to_string_lossy();
                    if !path_str.ends_with(".png")
                        && !path_str.ends_with(".jpg")
                        && !path_str.ends_with(".jpeg")
                        && !path_str.ends_with(".gif")
                        && !path_str.ends_with(".webp")
                    {
                        path.with_extension("png")
                    } else {
                        path.to_path_buf()
                    }
                }
            }
            None => PathBuf::from(filename),
        }
    }

    /// Save image bytes to file
    pub async fn save_image(&self, image_data: &[u8], path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                ImagoError::IoError(std::io::Error::other(format!(
                    "Failed to create directory: {}",
                    e
                )))
            })?;
        }

        let mut file = File::create(path).await?;
        file.write_all(image_data).await?;
        file.flush().await?;

        Ok(())
    }

    /// Display image in terminal
    pub fn display_in_terminal(&self, image_data: &[u8]) -> Result<()> {
        if !self.enable_preview {
            return Ok(());
        }

        // Prefer system `viu` preview because it renders correctly in user's Kitty setup.
        // Fallback to viuer when `viu` binary is unavailable.
        if Self::has_viu() {
            let tmp_path = std::env::temp_dir()
                .join(format!("imago_preview_{}.png", Self::generate_filename()));
            fs::write(&tmp_path, image_data)?;

            let mut cmd = Command::new("viu");
            cmd.arg("-w").arg(self.width.to_string());
            if let Some(h) = self.height {
                cmd.arg("-h").arg(h.to_string());
            }
            cmd.arg(&tmp_path);

            let status = cmd
                .status()
                .map_err(|e| ImagoError::DisplayError(format!("Failed to launch viu: {}", e)))?;

            let _ = fs::remove_file(&tmp_path);

            if !status.success() {
                return Err(ImagoError::DisplayError(
                    "viu preview process exited with non-zero status".to_string(),
                ));
            }

            return Ok(());
        }

        let img = image::load_from_memory(image_data)
            .map_err(|e| ImagoError::ImageError(format!("Failed to load image: {}", e)))?;

        let mut conf = Config {
            width: Some(self.width),
            height: self.height,
            ..Default::default()
        };

        let support = Self::detect_terminal_support();
        match support {
            TerminalSupport::Kitty => conf.use_kitty = true,
            TerminalSupport::ITerm2 => conf.use_iterm = true,
            _ => {}
        }

        print(&img, &conf)
            .map_err(|e| ImagoError::DisplayError(format!("Failed to display image: {}", e)))?;

        Ok(())
    }

    fn has_viu() -> bool {
        Command::new("viu").arg("--help").output().is_ok()
    }

    /// Print success message
    pub fn print_success(&self, path: &Path) {
        let path_str = path.display().to_string();
        println!("{} {}", "✅ Success!".green().bold(), "Saved to:".white());
        println!("   {}", path_str.cyan().underline());
    }

    /// Print generation started message
    pub fn print_generating(&self, prompt: &str) {
        println!("{} {}", "🎨 Generating:".blue().bold(), prompt.white());
    }

    /// Print error message
    pub fn print_error(&self, error: &ImagoError) {
        eprintln!("{} {}", "❌ Error:".red().bold(), error.to_string().red());
    }

    /// Print warning message
    pub fn print_warning(&self, message: &str) {
        println!("{} {}", "⚠️  Warning:".yellow(), message.yellow());
    }

    /// Detect terminal graphics support
    fn detect_terminal_support() -> TerminalSupport {
        if get_kitty_support() != KittySupport::None {
            return TerminalSupport::Kitty;
        }

        if is_iterm_supported() {
            return TerminalSupport::ITerm2;
        }

        if let Ok(term) = std::env::var("TERM") {
            if term.contains("kitty") || term.contains("wezterm") {
                return TerminalSupport::Kitty;
            }
        }

        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            if term_program == "iTerm.app" {
                return TerminalSupport::ITerm2;
            }
            if term_program == "WezTerm" {
                return TerminalSupport::Kitty;
            }
        }

        TerminalSupport::HalfBlocks
    }
}

#[derive(Debug, Clone, Copy)]
enum TerminalSupport {
    Kitty,
    ITerm2,
    HalfBlocks,
}
