use std::{convert::{TryFrom, TryInto}, rc::Rc};

use indexmap::{IndexMap, indexmap};
use priority_queue::PriorityQueue;

use crate::*;

#[derive(Clone, Debug)]
struct Edge {
	p1: NodeId,
	p2: NodeId,
	discriminator: Option<NodeId>,
	length: f64s,
	iidx: u64,
}
impl Edge {
	fn is_cycle(&self) -> bool {
		self.p1.as_ref() == self.p2.as_ref()
	}
	fn is_similar(&self, other: &Self) -> bool {
		self.p1 == other.p1 && self.p2 == other.p2 && self.discriminator == other.discriminator
	}
	fn dupe(&self) -> Self {
		Self {
			p1: self.p1.clone(),
			p2: self.p2.clone(),
			discriminator: self.discriminator.clone(),
			length: self.length,
			iidx: self.iidx+1,
		}
	}
	fn other(&self, n: &NodeId) -> &NodeId {
		if n.as_ref() == self.p1.as_ref() {
			&self.p2
		} else {
			&self.p1
		}
	}
	fn add(self, g: &mut Graph) -> Result<(), String> {
		let e = Rc::new(self);
		if !e.is_cycle() {
			g.get_mut(&e.p1).ok_or_else(|| format!("Nodes set missing {}", e.p1))?.push(e.clone());
		}
		g.get_mut(&e.p2).ok_or_else(|| format!("Nodes set missing {}", e.p2))?.push(e);
		Ok(())
	}
	fn remove(&self, g: &mut Graph){
		if !self.is_cycle() {
			if let Some(es) = g.get_mut(&self.p1) {
				es.retain(|e| e.as_ref() != self);
			}
		}
		if let Some(es) = g.get_mut(&self.p2) {
			es.retain(|e| e.as_ref() != self);
		}
	}
}
impl PartialEq<Edge> for Edge {
	fn eq(&self, other: &Self) -> bool {
		self.p1 == other.p1 && self.p2 == other.p2 && self.discriminator == other.discriminator && self.iidx == other.iidx
	}
}

type Graph = IndexMap<NodeId, Vec<Rc<Edge>>>;

type Path = Vec<Rc<Edge>>;

impl TryFrom<data::RoadGraph> for Graph {
	type Error = String;
	fn try_from(rs: data::RoadGraph) -> Result<Self, Self::Error> {
		let mut g: Graph = rs.nodes.into_iter().map(|n| (n.id, vec![])).collect();
		for r in rs.roads {
			Edge {
				p1: r.p1,
				p2: r.p2,
				discriminator: r.discriminator,
				length: r.distance,
				iidx: 0
			}.add(&mut g)?;
		}
		Ok(g)
	}
}

/// Make a graph eulirian by duplicating edges
fn kreek(mut g: Graph) -> Graph {
	for i in 0..g.len() {
		if let [e] = &g.get_index(i).unwrap().1[..] {
			e.dupe().add(&mut g).unwrap();
		}
	}
	while let Some(es) = g.values().find(|es| es.len() % 2 == 1) {
		let mut es: Vec<&Rc<Edge>> = es.iter().filter(|e| es.iter().filter(|ee| e.is_similar(ee)).count() == 1).collect();
		es.sort_unstable_by_key(|e| e.length);
		es.into_iter().next().unwrap().dupe().add(&mut g).unwrap();
	}
	g
}

/// Find shortest non-trivial undirected cycle on a vertex
fn dijkstra_on_a_bicycle(g: &Graph, n0: &NodeId) -> Option<Path> {
	let mut paths: IndexMap<NodeId, (f64s, Path)> = indexmap!{ n0.clone() => (f64s::INFINITY, vec![]) };
	let mut q: PriorityQueue<NodeId, f64s> = PriorityQueue::new();
	q.push(n0.clone(), f64s::INFINITY);
	while let Some((n, _)) = q.pop() {
		let (mut nd, np) = paths.get(&n).unwrap().clone();
		if &n == n0 {
			if nd < f64s::INFINITY {
				return Some(np.clone());
			} else {
				nd = f64s::try_from(0.0).unwrap();
			}
		}
		for e in g.get(&n).unwrap() {
			if !np.contains(e) {
				let v = e.other(&n);
				let vd = nd + e.length;
				if match paths.get(v) {
					Some((d, _)) => &vd < d,
					None => true
				} {
					let mut path = np.clone();
					path.push(e.clone());
					paths.insert(v.clone(), (vd, path));
					q.push(v.clone(), vd);
				}
			}
		}
	}
	None
}

fn graph_is_empty(g: &Graph) -> bool {
	g.values().all(Vec::is_empty)
}

fn path_length(path: &Path) -> f64s {
	path.iter().map(|e| e.length).sum()
}

fn path_shmlop<'a>(path: &'a Path, n0: &'a NodeId) -> Vec<(&'a NodeId, Option<&'a NodeId>)> {
	let mut vs = vec![(n0, None)];
	for e in path {
		vs.push((e.other(vs.last().unwrap().0), e.discriminator.as_ref()));
	}
	vs
}

/// Find list of cyclic paths over eulirian graph that together cover all edges starting/ending at specified vertices
fn bl33p(mut g: Graph, sns: &Vec<NodeId>) -> Vec<Path> {
	let mut cycles: Vec<Path> = sns.iter().map(|_| vec![]).collect();
	while cycles.len() > 0 && !graph_is_empty(&g) {
		let i = (0..sns.len()).min_by_key(|i| path_length(&cycles[*i])).unwrap();
		let n = &sns[i];
		let cycle = &mut cycles[i];
		if let Some((v, y)) = if cycle.len() > 0 {
			let shmlop = path_shmlop(cycle, n);
			(0..shmlop.len()).filter_map(|i| {
				let (v, _) = shmlop[i];
				if g.get(v).unwrap().len() > 0 {
					Some((v, i))
				} else {
					None
				}
			}).next()
		} else {
			Some((n, 0))
		} {
			let inj = dijkstra_on_a_bicycle(&g, v).unwrap();
			for e in &inj {
				e.remove(&mut g);
			}
			cycle.splice(y..y, inj);
		} else {
			panic!("The entirety of graph is not reachable from any of starting vertices");
		}
	}
	cycles
}

pub fn construct_flight_paths(roads: data::RoadGraph, drones: &data::Drones) -> Result<data::FlightPaths, String> {
	let sns: Vec<NodeId> = drones.iter().flat_map(|l| roads.locate(l)).collect();
	if sns.len() < drones.len() {
		return Err("Failed to locate positions to the road graph".to_string());
	}
	log::info!("Located drones");
	let mut g = kreek(roads.try_into()?);
	log::info!("Kreeked road graph");
	let cycles = bl33p(g, &sns);
	log::info!("Bleeped cycles");
	Ok(cycles.into_iter().zip(sns.into_iter()).map(|(path, n0)| path_shmlop(&path, &n0).into_iter().map(|(node, discriminator)| data::PathSegment { node: node.clone(), discriminator: discriminator.map(Clone::clone) }).collect()).collect())
}
