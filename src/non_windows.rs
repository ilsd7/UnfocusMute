use crate::i18n::Language;

pub fn run() -> anyhow::Result<()> {
    println!("{}", Language::default().strings().unsupported_os);
    Ok(())
}
