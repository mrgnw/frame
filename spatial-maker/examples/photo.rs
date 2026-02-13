//! Example CLI for processing a single photo to spatial photo with multiple output formats
//!
//! Usage:
//!   cargo run --example photo -- --input input.jpg --output spatial.jpg --encoder s
//!   cargo run --example photo -- --input input.jpg --output spatial.jpg --format top-bottom
//!   cargo run --example photo -- --input input.jpg --output spatial.png --format png

use clap::Parser;
use spatial_maker::{
    process_photo, ImageEncoding, MVHEVCConfig, OutputFormat, OutputOptions, SpatialConfig,
};
use std::path::PathBuf;
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "spatial-maker photo")]
#[command(about = "Convert a single photo to spatial photo (multiple output formats)", long_about = None)]
struct Args {
    /// Input image file path
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output image file path
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,

    /// Depth model encoder size: s (small), b (base), or l (large)
    #[arg(short, long, default_value = "s")]
    encoder: String,

    /// Maximum disparity for stereo generation (pixels)
    #[arg(long, default_value = "30")]
    max_disparity: u32,

    /// Target input size for depth model (shorter side)
    #[arg(long, default_value = "518")]
    target_size: u32,

    /// Output stereo format: side-by-side, top-bottom, or separate
    #[arg(short, long, default_value = "side-by-side")]
    format: String,

    /// Output image encoding: jpeg or png
    #[arg(long, default_value = "jpeg")]
    image_format: String,

    /// JPEG quality (1-100, only used with jpeg format)
    #[arg(long, default_value = "95")]
    quality: u8,

    /// Encode to MV-HEVC format using spatial CLI
    #[arg(long)]
    mvhevc: bool,

    /// Path to spatial CLI tool (only used with --mvhevc)
    #[arg(long)]
    spatial_path: Option<PathBuf>,

    /// MV-HEVC quality (0-100, only used with --mvhevc)
    #[arg(long, default_value = "95")]
    mvhevc_quality: u8,

    /// Keep intermediate stereo image after MV-HEVC encoding
    #[arg(long)]
    keep_intermediate: bool,

    /// Disable CoreML (use CPU on macOS)
    #[arg(long)]
    no_coreml: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    // Validate input file exists
    if !args.input.exists() {
        eprintln!("❌ Input file not found: {:?}", args.input);
        std::process::exit(1);
    }

    // Parse output stereo format
    let layout = match args.format.to_lowercase().as_str() {
        "side-by-side" | "sbs" => OutputFormat::SideBySide,
        "top-bottom" | "tb" => OutputFormat::TopAndBottom,
        "separate" => OutputFormat::Separate,
        other => {
            eprintln!(
                "❌ Invalid format '{}'. Use: side-by-side, top-bottom, or separate",
                other
            );
            std::process::exit(1);
        }
    };

    // Parse image encoding
    let image_encoding = match args.image_format.to_lowercase().as_str() {
        "png" => ImageEncoding::Png,
        "jpeg" | "jpg" => {
            let quality = args.quality.max(1).min(100);
            ImageEncoding::Jpeg { quality }
        }
        other => {
            eprintln!("❌ Invalid image format '{}'. Use: jpeg or png", other);
            std::process::exit(1);
        }
    };

    // Create spatial config
    let spatial_config = SpatialConfig {
        encoder_size: args.encoder.clone(),
        max_disparity: args.max_disparity,
        target_depth_size: args.target_size,
        use_coreml: !args.no_coreml,
    };

    // Create output options
    let mvhevc = if args.mvhevc {
        Some(MVHEVCConfig {
            spatial_cli_path: args.spatial_path,
            enabled: true,
            quality: args.mvhevc_quality,
            keep_intermediate: args.keep_intermediate,
        })
    } else {
        None
    };

    let output_options = OutputOptions {
        layout,
        image_format: image_encoding,
        mvhevc,
    };

    // Print summary
    println!("🎬 Processing photo...");
    println!("  Input:       {:?}", args.input);
    println!("  Output:      {:?}", args.output);
    println!("  Format:      {}", layout.name());
    println!(
        "  Encoder:     {} (CoreML: {})",
        spatial_config.encoder_size, spatial_config.use_coreml
    );
    println!("  Disparity:   {} px", spatial_config.max_disparity);
    if args.mvhevc {
        println!("  MV-HEVC:     enabled (quality: {})", args.mvhevc_quality);
    }

    // Run processing
    match process_photo(&args.input, &args.output, spatial_config, output_options).await {
        Ok(_) => {
            println!("✅ Photo processing complete!");
            println!("   Saved to: {:?}", args.output);
        }
        Err(e) => {
            eprintln!("❌ Error processing photo: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
