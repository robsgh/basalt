use anyhow::Result;
use basalt::app::BasaltApp;

fn main() -> Result<()> {
    env_logger::init();

    log::debug!("Starting Basalt...");
    let mut app = BasaltApp::default();

    app.run()
}
