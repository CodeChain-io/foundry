extern crate fml_tests;

#[cfg(all(unix, target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    let args = std::env::args().collect();
    fml_tests::mod_relayer_main(args);
    Ok(())
}
