use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use calculator_render::View;
use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let state = output.join(format!("calculator-smoke-{stamp}"));
    fs::create_dir(&state)?;
    let mut app = View::open(state.clone());
    write_png(
        &output.join("calculator.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;
    for (row, column) in [(4, 1), (3, 0), (3, 3), (4, 1), (3, 1), (4, 2)] {
        let x = 100.0 + 300.0 * column as f32;
        let y = 825.0 + 167.0 * row as f32;
        assert_eq!(app.input(Input::Tap(x, y)), Change::Partial);
    }
    assert_eq!(app.calculator.entry(), "0.3");
    assert_eq!(app.input(Input::Tap(900.0, 620.0)), Change::Partial);
    assert_eq!(fs::read_to_string(state.join("result.txt"))?, "0.3\n");
    assert_eq!(View::open(state.clone()).calculator.entry(), "0.3");
    write_png(
        &output.join("calculator-copy-dark.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, true),
    )?;
    assert_eq!(app.input(Input::Tap(300.0, 620.0)), Change::Full);
    write_png(
        &output.join("calculator-history.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;
    assert_eq!(app.input(Input::PageBack), Change::Full);
    let tap = |app: &mut View, x, y, expected| assert_eq!(app.input(Input::Tap(x, y)), expected);
    let mut tool_capture = |app: &View, name: &str, dark| {
        write_png(
            &output.join(name),
            WIDTH,
            HEIGHT,
            &frame(app, &mut canvas, dark),
        )
    };
    tap(&mut app, 600.0, 615.0, Change::Full);
    tap(&mut app, 620.0, 275.0, Change::Full);
    tap(&mut app, 620.0, 627.0, Change::Partial);
    tool_capture(&app, "calculator-convert.png", false)?;
    assert_eq!(app.input(Input::PageBack), Change::Full);
    tap(&mut app, 620.0, 470.0, Change::Full);
    tap(&mut app, 620.0, 744.0, Change::Partial);
    tap(&mut app, 920.0, 1370.0, Change::Partial);
    tap(&mut app, 920.0, 1370.0, Change::Partial);
    tool_capture(&app, "calculator-steps.png", false)?;
    assert_eq!(app.input(Input::PageBack), Change::Full);
    tap(&mut app, 620.0, 674.0, Change::Full);
    tap(&mut app, 156.0, 449.0, Change::Partial);
    tap(&mut app, 750.0, 449.0, Change::Partial);
    tool_capture(&app, "calculator-graph-dark.png", true)?;
    tap(&mut app, 450.0, 449.0, Change::Partial);
    assert_eq!(app.input(Input::PageBack), Change::Full);
    assert_eq!(app.input(Input::PageBack), Change::Full);
    assert_eq!(app.calculator.entry(), "0.3");
    fs::remove_file(state.join("calculator.json"))?;
    fs::remove_file(state.join("result.txt"))?;
    fs::remove_dir(state)?;
    Ok(())
}
