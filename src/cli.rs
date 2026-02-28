use clap::{Parser, ValueHint};
use std::path::PathBuf;

/// Imago - High-performance CLI image generator using Gemini Image Generation API
#[derive(Parser, Debug)]
#[command(
    name = "imago",
    version = env!("CARGO_PKG_VERSION"),
    author = "Imago Contributors",
    about = "Generate images using Gemini Image Generation API with instant terminal preview",
    long_about = r#"
Imago is a high-performance CLI tool that generates images using the Gemini Image Generation API.
It provides instant terminal preview using modern terminal graphics protocols.

EXAMPLES:
    imago "a beautiful sunset over mountains"
    imago "cyberpunk city at night" -o ./images/
    imago "abstract art" --width 80 --no-preview

ENVIRONMENT:
    GEMINI_API_KEY    Required. Your Google Gemini API key.
"#
)]
pub struct Cli {
    /// The prompt describing the image to generate
    #[arg(value_name = "PROMPT", help = "Description of the image to generate")]
    pub prompt: String,

    /// Output directory or file path
    #[arg(
        short = 'o',
        long = "output",
        value_name = "PATH",
        value_hint = ValueHint::DirPath,
        help = "Output directory or file path for the generated image"
    )]
    pub output: Option<PathBuf>,

    /// Preview width in terminal columns
    #[arg(
        short = 'w',
        long = "width",
        value_name = "COLUMNS",
        default_value = "60",
        help = "Width of the preview in terminal columns"
    )]
    pub width: u32,

    /// Preview height in terminal rows (optional)
    #[arg(
        short = 'H',
        long = "height",
        value_name = "ROWS",
        help = "Height of the preview in terminal rows (optional)"
    )]
    pub height: Option<u32>,

    /// Disable terminal preview
    #[arg(
        long = "no-preview",
        help = "Disable terminal preview after generation"
    )]
    pub no_preview: bool,

    /// Model to use for generation
    #[arg(
        short = 'm',
        long = "model",
        value_name = "MODEL",
        default_value = "gemini-2.5-flash-image",
        help = "Gemini model to use for image generation"
    )]
    pub model: String,

    /// API key (overrides environment variable)
    #[arg(
        short = 'k',
        long = "api-key",
        value_name = "KEY",
        help = "Gemini API key (overrides GEMINI_API_KEY environment variable)"
    )]
    pub api_key: Option<String>,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose", help = "Enable verbose output")]
    pub verbose: bool,

    /// Disable color output
    #[arg(long = "no-color", help = "Disable colored output")]
    pub no_color: bool,
}

impl Cli {
    /// Validate CLI arguments
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.width == 0 {
            return Err(crate::error::ImagoError::ResponseFormatError {
                message: "Width must be greater than 0".to_string(),
            });
        }
        Ok(())
    }
}
