mod commands;
mod config;
mod keys;
mod layout;
mod log;
mod state;
mod wm;

fn main() {
    log::init();
    if let Err(error) = wm::run() {
        ::log::error!("fatal: {error:#}");
        std::process::exit(1);
    }
}
