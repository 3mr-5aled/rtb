use anyhow::Result;
use rtb::engine::RtbEngine;

fn main() -> Result<()> {
    color_eyre::install().ok();

    let exit_code = RtbEngine::dispatch()?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}
