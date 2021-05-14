use std::borrow::Cow;

mod f64nn;
use clap::{App, Arg, SubCommand, crate_version};
use f64nn::*;
mod data;
mod brr;
mod gj;

pub type NodeId = Cow<'static, str>;

fn main() -> std::io::Result<()> {
	env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));
	let matches = App::new("Flight Paths Compute")
							.version(crate_version!())
							.about("Make it fly!")
							.subcommand(SubCommand::with_name("fly")
								.about("Compute flight paths")
								.arg(Arg::with_name("road-graph")
										.takes_value(true)
										.required(true)
										.index(1)
										.help("Road Graph JSON"))
								.arg(Arg::with_name("drones")
										.takes_value(true)
										.required(true)
										.index(2)
										.help("Drones configuration JSON"))
								.arg(Arg::with_name("output")
										.takes_value(true)
										.required(true)
										.index(3)
										.help("Output JSON"))
							)
							.subcommand(SubCommand::with_name("geojson")
								.about("Convert computed flight paths into distinct GeoJSONs")
								.arg(Arg::with_name("road-graph")
										.takes_value(true)
										.required(true)
										.index(1)
										.help("Road Graph JSON"))
								.arg(Arg::with_name("paths")
										.takes_value(true)
										.required(true)
										.index(2)
										.help("(Produced) Flight Paths"))
								.arg(Arg::with_name("prefix")
										.takes_value(true)
										.required(true)
										.index(3)
										.help(r#"GeoJSON files prefix - the generated files will be named "{prefix}.{index}.geojson""#))
							)
							.get_matches();
	if let Some(matches) = matches.subcommand_matches("fly") {
		log::info!("Loading...");
		log::trace!("tracing enabled");
		let drones: data::Drones = serde_json::from_reader(&std::fs::File::open(matches.value_of("drones").unwrap())?).expect("Drones config invalid JSON");
		let roads: data::RoadGraph = serde_json::from_reader(&std::fs::File::open(matches.value_of("road-graph").unwrap())?).expect("Road graph invalid JSON");
		log::info!("Loaded configuration");
		let paths = brr::construct_flight_paths(roads, &drones).unwrap();
		log::info!("Constructed paths");
		serde_json::to_writer(&std::fs::File::create(matches.value_of("output").unwrap())?, &paths).unwrap();
	} else if let Some(matches) = matches.subcommand_matches("geojson") {
		let roads: data::RoadGraphNodes = serde_json::from_reader(&std::fs::File::open(matches.value_of("road-graph").unwrap())?).expect("Road graph config invalid JSON");
		let paths: data::FlightPaths = serde_json::from_reader(&std::fs::File::open(matches.value_of("paths").unwrap())?).expect("Flight paths invalid JSON");
		let pref = matches.value_of("prefix").unwrap();
		log::info!("Loaded configuration");
		let g = gj::roads_to_nodes(roads);
		for (i, path) in (0..paths.len()).zip(paths.into_iter()) {
			serde_json::to_writer(&std::fs::File::create(format!("{}.{}.geojson", pref, i))?, &gj::path_to_geojson(&g, path)).unwrap();
		}
	}
	Ok(())
}
