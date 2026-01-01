mod wm;
mod log;
mod state;
mod layout;
mod keys;

fn main() -> anyhow::Result<()> {
    log::init();
    wm::run()
}
