use anyhow::Context;
use eccodes::{CodesFile, FallibleIterator, ProductKind};
fn main() -> anyhow::Result<()> {
let mut handle = CodesFile::new_from_file("./data/iceland.grib", ProductKind::GRIB)?;
let mut current_message = handle.ref_message_iter().next()?.context("no message")?;

let mut keys_iter = current_message.default_keys_iterator()?;

while let Some(key_name) = keys_iter.next()? {
    println!("{key_name}");
}
Ok(())
}
