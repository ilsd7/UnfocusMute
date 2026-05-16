mod audio;
mod process;
mod startup;
mod ui;

pub fn run() -> anyhow::Result<()> {
    ui::run()
}
