use eccodes::{CodesError, CodesFile, FallibleIterator, KeyRead, ProductKind};
use std::fs::remove_file;
fn main() -> anyhow::Result<(), CodesError> {
    let mut handle = CodesFile::new_from_file("./data/iceland-levels.grib", ProductKind::GRIB)?;

    while let Some(msg) = handle.ref_message_iter().next()? {
        let level: i64 = msg.read_key("level")?;
        if level == 800 {
            msg.write_to_file("./data/iceland-800hPa.grib", true)?;
        }
    }
    remove_file("./data/iceland-800hPa.grib")?;
    Ok(())
}
