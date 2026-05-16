use crate::i18n::Language;

pub fn run() -> anyhow::Result<()> {
    println!("{}", Language::Ko.strings().unsupported_os);
    Ok(())
}
