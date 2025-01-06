//====================================================================

use hecs_game::run;

//====================================================================

fn main() {
    env_logger::Builder::new()
        .filter_module("hecs_game", log::LevelFilter::Trace)
        .filter_module("hecs_engine", log::LevelFilter::Trace)
        //
        .filter_module("engine", log::LevelFilter::Trace)
        .filter_module("pipelines", log::LevelFilter::Trace)
        .filter_module("renderer", log::LevelFilter::Trace)
        .filter_module("common", log::LevelFilter::Trace)
        //
        .filter_module("winit", log::LevelFilter::Trace)
        .filter_module("wgpu", log::LevelFilter::Warn)
        .init();

    log::info!("Hello World");

    run()
}

//====================================================================
