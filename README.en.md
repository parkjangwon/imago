# imago

[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![CLI](https://img.shields.io/badge/Type-CLI-blue)](#)

A Rust CLI image generator powered by the Gemini Image Generation API, with optional terminal preview.

## Features
- Prompt-based image generation
- Custom output path support
- Terminal image preview (`viuer`)
- Automatic fallback when a model returns 404
- Single-binary CLI workflow

## Requirements
- Rust (stable)
- Gemini API key

Environment variable:
```bash
export GEMINI_API_KEY="your_api_key"
```

## Installation
```bash
curl -fsSL https://raw.githubusercontent.com/parkjangwon/imago/main/install.sh | bash
```

## Build
```bash
cd /Users/pjw/dev/project/imago
cargo build --release
```

Binary:
```bash
./target/release/imago
```

## Quick Start
```bash
imago "a cinematic blue cyberpunk city at night"
```

Save to a directory:
```bash
imago "minimal blue abstract geometric wallpaper" -o ./output/
```

Disable preview:
```bash
imago "product mockup on white desk" --no-preview
```

Specify model (optional):
```bash
imago "futuristic interface concept" --model gemini-2.5-flash-image
```

## CLI Options
```text
Usage: imago [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>                     Prompt describing the image to generate

Options:
  -o, --output <PATH>          Output file or directory path
  -w, --width <COLUMNS>        Terminal preview width (default: 60)
  -H, --height <ROWS>          Terminal preview height (optional)
      --no-preview             Disable terminal preview
  -m, --model <MODEL>          Gemini model to use
                                (default: gemini-2.5-flash-image)
  -k, --api-key <KEY>          API key override (higher priority than env)
  -v, --verbose                Verbose output
      --no-color               Disable colored output
  -h, --help                   Help
  -V, --version                Version
```

## Model Fallback
If the requested model returns 404, imago retries with fallback models in order:
- gemini-2.5-flash-image
- gemini-3.1-flash-image-preview
- gemini-3-pro-image-preview
- gemini-2.0-flash-exp-image-generation

## Troubleshooting
### 1) `GEMINI_API_KEY` error
- Message: `API key not found`
- Fix: set `GEMINI_API_KEY` or pass `--api-key`

### 2) Generation failure (model/access)
- Check API key permissions, quota/billing, and model availability
- Retry with an explicit `--model`

### 3) No terminal preview
- Your terminal may not support image protocols
- Use `--no-preview` to generate only

## License
MIT
