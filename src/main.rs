use std::borrow::Cow;

mod f64nn;
use clap::{App, Arg};
use f64nn::*;
mod data;
mod brr;

pub type NodeId = Cow<'static, str>;

fn main() -> std::io::Result<()> {
	env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));
	let matches = App::new("Flight Paths Compute")
							.about("Make it fly!")
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
							.get_matches();
	log::info!("Loading...");
	log::trace!("tracing enabled");
	let drones: data::Drones = serde_json::from_reader(&std::fs::File::open(matches.value_of("drones").unwrap())?).expect("Drones config invalid JSON");
	let roads: data::RoadGraph = serde_json::from_reader(&std::fs::File::open(matches.value_of("road-graph").unwrap())?).expect("Drones config invalid JSON");
	log::info!("Loaded configuration");
	let paths = brr::construct_flight_paths(roads, &drones).unwrap();
	log::info!("Constructed paths");
	serde_json::to_writer(&std::fs::File::create(matches.value_of("output").unwrap())?, &paths).unwrap();
	Ok(())
}
