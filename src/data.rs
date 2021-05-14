use crate::*;

use serde::*;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RoadSegment {
	pub p1: NodeId,
	pub p2: NodeId,
	pub discriminator: Option<NodeId>,
	pub distance: f64s,
	pub sidewalks: (bool, bool),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Node {
	pub id: NodeId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoadGraph {
	pub roads: Vec<RoadSegment>,
	pub nodes: Vec<Node>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum Location {
	Coordinates(f64, f64),
	Node(NodeId),
}

pub type Drones = Vec<Location>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PathSegment {
	pub node: NodeId,
	pub discriminator: Option<NodeId>,
}

pub type FlightPath = Vec<Vec<PathSegment>>;
