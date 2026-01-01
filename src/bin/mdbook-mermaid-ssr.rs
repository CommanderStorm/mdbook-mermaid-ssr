use clap::{Parser, Subcommand};
use mdbook_mermaid_ssr::{Mermaid, config::Config};
use mdbook_preprocessor::Preprocessor;
use mdbook_preprocessor::errors::Error;

use std::{io, process};

#[derive(Parser)]
#[command(
    name = "mdbook-mermaid-ssr",
    version,
    about = "mdbook preprocessor to add mermaid support with server-side rendering"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check whether a renderer is supported by this preprocessor
    Supports {
        /// The renderer to check support for
        renderer: String,
    },
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Supports { renderer }) => handle_supports(&renderer),
        None => {
            if let Err(e) = handle_preprocessing() {
                log::error!("Cannot preprocess mermaid diagrams because {e}");
                process::exit(1);
            }
        }
    }
}

fn handle_preprocessing() -> Result<(), Error> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
        log::error!(
            "Warning: The mdbook-mermaid preprocessor was built against version \
             {our_ver} of mdbook, but we're being called from version {ctx_ver}",
            our_ver = mdbook_preprocessor::MDBOOK_VERSION,
            ctx_ver = ctx.mdbook_version
        );
    }

    let config = Config::from_context(&ctx);
    let preprocessor = Mermaid::new(config)
        .map_err(|e| Error::msg(format!("Failed to initialize mermaid preprocessor: {}", e)))?;
    let processed_book = preprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(renderer: &str) -> ! {
    // For the supports check, we don't need to parse config, just check if we can initialize
    let preprocessor = match Mermaid::new(Config::default()) {
        Ok(p) => p,
        Err(_) => {
            // If we can't initialize, we can't support any renderer
            process::exit(1);
        }
    };
    let supported = preprocessor.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if let Ok(true) = supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
