#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(e) = run() {
        eprintln!("asset-export: {e}");
        std::process::exit(1);
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let value = |key: &str| {
        args.iter()
            .position(|a| a == key)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    if args.iter().any(|a| a == "--help") {
        println!("asset-export [--model NAME] [--seed N] [--output tmp/glb]");
        return Ok(());
    }
    let options = models::GenerationOptions {
        seed: value("--seed")
            .map(|s| s.parse())
            .transpose()?
            .unwrap_or(models::DEFAULT_SEED),
    };
    let output = value("--output").unwrap_or("tmp/glb".into());
    let names = value("--model")
        .map(|n| vec![n])
        .unwrap_or_else(|| models::NAMES.iter().map(|n| n.to_string()).collect());
    let models: Vec<_> = names
        .iter()
        .map(|name| models::generate(name, options))
        .collect::<Result<_, _>>()?;
    for model in models {
        let path = std::path::Path::new(&output).join(format!("{}.glb", model.name));
        models::export::write(&model, &path)?;
        println!("{}", path.display());
    }
    Ok(())
}
#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("GLB filesystem export is native-only");
}
