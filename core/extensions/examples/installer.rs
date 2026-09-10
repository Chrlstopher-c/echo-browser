//! Installe une extension du catalogue Chrome et affiche l'inventaire.
//!
//! Usage : cargo run -p echo-extensions --example installer -- <identifiant ou adresse>

use echo_extensions::Extensions;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let root = std::env::var("ECHO_EXT_DIR").unwrap_or_else(|_| "data/extensions".to_string());
    let extensions = Extensions::new(&root);

    if let Some(target) = std::env::args().nth(1) {
        let id = extensions.install(&target)?;
        println!("declaree : {id} — Chromium l'installe au prochain demarrage");
    }

    println!("\n== INVENTAIRE ==");
    for extension in extensions.list() {
        let state = if extension.enabled { "active " } else { "inactive" };
        println!("  {state}  {:<34} v{:<12} {}", extension.name, extension.version, extension.id);
    }
    println!("\n== CHARGEES AU DEMARRAGE ==");
    for path in extensions.loadable() {
        println!("  {path}");
    }
    Ok(())
}
