use anyhow::Result;
use basalt::app::BasaltApp;

fn main() -> Result<()> {
    let mut app = BasaltApp::default();

    app.run()
}
