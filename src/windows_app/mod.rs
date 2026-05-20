mod audio;
pub(crate) mod error;
mod process;
mod startup;
mod ui;

pub fn run() -> error::Result<()> {
    ui::run()
}
