use anyhow::Result;
use custom_rust_dist::cli;

fn main() -> Result<()> {
    let cli = cli::parse_cli();
    cli.execute()
}
