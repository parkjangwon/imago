mod cli;
mod error;
mod gemini;
mod image_handler;

use crate::cli::Cli;
use crate::error::{ImagoError, Result};
use crate::gemini::GeminiClient;
use crate::image_handler::ImageHandler;
use clap::Parser;
use colored::control;
use std::env;

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Setup colored output
    if cli.no_color {
        control::set_override(false);
    }

    // Validate arguments
    if let Err(e) = cli.validate() {
        let handler = ImageHandler::new(cli.width, cli.height, false);
        handler.print_error(&e);
        std::process::exit(1);
    }

    // Run the application
    if let Err(e) = run(cli).await {
        let handler = ImageHandler::new(60, None, false);
        handler.print_error(&e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Get API key
    let api_key = cli
        .api_key
        .or_else(|| env::var("GEMINI_API_KEY").ok())
        .ok_or(ImagoError::MissingApiKey)?;

    if cli.verbose {
        println!("Using model: {}", cli.model);
    }

    // Create components
    let client = GeminiClient::new(api_key, cli.model.clone());
    let handler = ImageHandler::new(cli.width, cli.height, !cli.no_preview);

    // Print generation message
    handler.print_generating(&cli.prompt);

    // Generate image
    let (image_data, _) = client.generate_image(&cli.prompt).await?;

    if cli.verbose {
        println!("Image generated: {} bytes", image_data.len());
    }

    // Resolve output path
    let output_path = handler.resolve_output_path(cli.output.as_deref());

    // Save the image
    handler.save_image(&image_data, &output_path).await?;

    // Print success message
    handler.print_success(&output_path);

    // Display in terminal
    if !cli.no_preview {
        println!();
        match handler.display_in_terminal(&image_data) {
            Ok(_) => {}
            Err(e) => {
                handler.print_warning(&format!("Could not display preview: {}", e));
            }
        }
    }

    Ok(())
}
