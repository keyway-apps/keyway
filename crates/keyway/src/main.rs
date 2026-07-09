use anyhow::Result;
use clap::Parser;
use gpui::Application;
use gpui_platform;

use crate::cli::Cli;

mod cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(_cmd) = cli.command {
        return Ok(());
    }

    ktracing::init();

    let app = Application::with_platform(gpui_platform::current_platform(false));

    app.run(move |cx| {
        i18n::init("en");
        gpui_tokio::init(cx);

        
        workspace::init(cx);
    });

    Ok(())
}
