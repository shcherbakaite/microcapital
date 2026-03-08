#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "MicroCapital",
        options,
        Box::new(|cc| Ok(Box::new(microcapital::MicroCapitalApp::from_storage(cc.storage)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}
