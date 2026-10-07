//! Chargement du decodeur installe : le moteur lie `libffmpeg.so` des le lancement, avant `main`. Le seul moyen de lui
//! en donner un autre est de repartir avec son dossier en tete de `LD_LIBRARY_PATH` (prioritaire sur le dossier du
//! moteur). Les processus enfants heritent de l'environnement.

use std::os::unix::process::CommandExt;

/// A appeler en tout premier dans `main`. Ne revient pas si le processus est relance.
pub fn adopt_installed() {
    if std::env::args().any(|a| a.starts_with("--type=")) || !super::pack::installed() {
        return;
    }
    let dir = super::pack::dir().to_string_lossy().into_owned();
    let current = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
    if current.split(':').next() == Some(dir.as_str()) {
        return;
    }
    let value = if current.is_empty() { dir } else { format!("{dir}:{current}") };
    let program = match std::env::current_exe() {
        Ok(program) => program,
        Err(err) => return eprintln!("decodeur complet ignore : executable introuvable ({err})"),
    };
    let err = std::process::Command::new(program)
        .args(std::env::args_os().skip(1))
        .env("LD_LIBRARY_PATH", value)
        .exec();
    eprintln!("decodeur complet ignore : relance impossible ({err})");
}
