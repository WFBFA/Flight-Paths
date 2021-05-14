use crate::*;
use data::*;

use geojson::*;
use indexmap::IndexMap;

pub type Nodes = IndexMap<NodeId, Node>;

pub fn roads_to_nodes(g: RoadGraphNodes) -> Nodes {
	g.nodes.into_iter().map(|n| (n.id.clone(), n)).collect()
}

pub fn path_to_geojson(g: &Nodes, path: Vec<PathSegment>) -> Geometry {
	Geometry::new(Value::LineString(path.into_iter().flat_map(|PathSegment { node, .. }| g.get(&node).map(|node| vec![node.coordinates.0, node.coordinates.1])).collect()))
}
