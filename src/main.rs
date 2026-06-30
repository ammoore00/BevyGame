use std::env;
use std::fs::{create_dir_all, File};
use std::io::Write;
use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy_mod_debugdump::schedule_graph;
use app::AppPlugin;

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins(AppPlugin);

    let args: Vec<String> = env::args().collect();

    #[cfg(feature = "dev")]
    if args.iter().any(|arg| arg == "--schedule") {

        let dot_string = bevy_mod_debugdump::schedule_graph_dot(&mut app, Update, &schedule_graph::Settings::default());

        let svg_data = generate_svg_data(dot_string).expect("Failed to generate SVG data");

        create_dir_all("debug").expect("Unable to create debug directory");
        let mut file = File::create("debug/schedule.svg").expect("Unable to create SVG file");
        file.write_all(svg_data.as_slice()).expect("Unable to write SVG data");

        println!("Successfully generated schedule.svg!");
        return AppExit::Success;
    }

    // Otherwise, run the game normally
    app.run()
}

#[cfg(feature = "dev")]
fn generate_svg_data(dot_string: String) -> Result<Vec<u8>, std::io::Error> {
    use graphviz_rust::printer::PrinterContext;
    use graphviz_rust::cmd::Format;

    let graph = graphviz_rust::parse(dot_string.as_str()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    graphviz_rust::exec(
        graph,
        &mut PrinterContext::default(),
        vec![Format::Svg.into()],
    )
}