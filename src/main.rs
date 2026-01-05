mod keys;
mod layout;
mod log;
mod state;
mod wm;

fn main() -> anyhow::Result<()> {
    log::init();
    wm::run()
}
