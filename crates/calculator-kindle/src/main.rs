use std::path::PathBuf;

use calculator_render::View;

fn main() -> Result<(), String> {
    let directory = std::env::var_os("PRIMITIVE_APP_DIR")
        .map_or_else(|| PathBuf::from("/mnt/us/calculator"), PathBuf::from);
    let calculator = View::open(directory.clone());
    primitive_app::run(calculator, &directory)
}
