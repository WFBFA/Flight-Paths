use std::{cmp::max, collections::{HashMap, HashSet}, convert::{TryFrom, TryInto}, rc::Rc};

use indexmap::IndexMap;
use priority_queue::PriorityQueue;

use crate::*;

#[derive(Clone, Eq, Debug)]
struct Edge {
	p1: NodeId,
	p2: NodeId,
	discriminator: Option<NodeId>,
	directed: bool,
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
			directed: self.directed,
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
		g.get_mut(&e.p1).ok_or_else(|| format!("Nodes set missing {}", e.p1))?.push(e.clone());
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
impl std::hash::Hash for Edge {
	fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
		(&self.p1, &self.p2, self.discriminator.as_ref(), self.iidx).hash(h)
	}
}
impl std::fmt::Display for Edge {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({} - {} v{} #{}# {:.1}m)", self.p1, self.p2, self.discriminator.as_ref().map(|s| s.as_ref()).unwrap_or("-"), self.iidx, self.length.f())
	}
}

type Graph = IndexMap<NodeId, Vec<Rc<Edge>>>;

type Path = Vec<Rc<Edge>>;

impl TryFrom<data::RoadGraph> for Graph {
	type Error = String;
	fn try_from(rs: data::RoadGraph) -> Result<Self, Self::Error> {
		let mut g: Graph = rs.nodes.nodes.into_iter().map(|n| (n.id, vec![])).collect();
		for r in rs.roads {
			Edge {
				p1: r.p1,
				p2: r.p2,
				discriminator: r.discriminator,
				directed: r.directed,
				length: r.distance,
				iidx: 0
			}.add(&mut g)?;
		}
		Ok(g)
	}
}

/// Calculate combined degree of a vertex
fn combined_degree<const DIRESPECT: bool>(n: &NodeId, edges: &Vec<Rc<Edge>>) -> i64 {
	if DIRESPECT {
		-(edges.iter().filter(|e| e.directed && &e.p1 == n).count() as i64 - edges.iter().filter(|e| e.directed && &e.p2 == n).count() as i64).abs() + edges.iter().filter(|e| !e.directed).count() as i64
	} else {
		edges.len() as i64
	}
}

/// Check whether the given node does not prevent a graph from being eulirian
fn eulirian_compatible<const DIRESPECT: bool>(n: &NodeId, edges: &Vec<Rc<Edge>>) -> bool {
	let d = combined_degree::<DIRESPECT>(n, edges);
	d % 2 == 0 && (!DIRESPECT || d >= 0)
}

/// Pick the best edge to augment
fn kreek_pick<'a, const DIRESPECT: bool>(g: &Graph, n: &NodeId, es: &'a Vec<Rc<Edge>>) -> &'a Rc<Edge> {
	let epre = es.iter().filter(|e| !e.is_cycle() && es.iter().filter(|ee| e.is_similar(ee)).count() == 1);
	if DIRESPECT {
		let ind = es.iter().filter(|e| e.directed && &e.p2 == n).count();
		let outd = es.iter().filter(|e| e.directed && &e.p1 == n).count();
		let mut es: Vec<&Rc<Edge>> = epre.filter(|e| !e.directed || (outd > ind && &e.p2 == n) || (ind > outd && &e.p1 == n)).collect();
		es.sort_unstable_by_key(|e| (-((g.get(&e.p1).unwrap().len()%2 + g.get(&e.p2).unwrap().len()%2) as i64), e.length)); //TODO we can do better!
		es
	} else {
		let mut es: Vec<&Rc<Edge>> = epre.collect();
		es.sort_unstable_by_key(|e| (-((g.get(&e.p1).unwrap().len()%2 + g.get(&e.p2).unwrap().len()%2) as i64), e.length));
		es
	}.into_iter().next().unwrap()
}

/// Make a graph eulirian by duplicating edges
fn kreek<const DIRESPECT: bool>(mut g: Graph) -> Result<Graph, String> {
	let es0 = graph_edges(&g);
	log::trace!("kreek kreek started on -> {}|{}", g.len(), es0);
	for i in 0..g.len() {
		if let [e] = &g.get_index(i).unwrap().1[..] {
			if DIRESPECT && e.directed && &e.p2 == g.get_index(i).unwrap().0 {
				return Err(format!("Can not make augment a directed graph to Eulirian when it has one-way stumbles ({:?})", e));
			}
			e.dupe().add(&mut g).unwrap();
		}
	}
	let es1 = graph_edges(&g);
	log::trace!("duped {} singular edges -> {}|{}", es1-es0, g.len(), es1);
	while let Some((n, es)) = g.iter().find(|(n, es)| !eulirian_compatible::<DIRESPECT>(n, es)) {
		kreek_pick::<DIRESPECT>(&g, n, es).dupe().add(&mut g).unwrap();
	}
	let es2 = graph_edges(&g);
	log::trace!("duped {} additional edges -> {}|{}", es2-es1, g.len(), es2);
	Ok(g)
}

/// Find shortest non-trivial cycle on a vertex
fn bicycle<const DIRESPECT: bool>(g: &Graph, n0: &NodeId, pred: Option<&dyn Fn(&Rc<Edge>) -> bool>) -> Option<Path> {
	log::trace!("cycling on {} d {}", n0, combined_degree::<DIRESPECT>(n0, g.get(n0).unwrap()));
	let mut q: PriorityQueue<(NodeId, Path), f64s> = PriorityQueue::new();
	q.push((n0.clone(), vec![]), f64s::ZERO);
	while let Some(((n, path), d)) = q.pop() {
		// log::trace!("{} {}", n, path.len());
		if &n == n0 && path.len() > 0 {
			return Some(path);
		}
		for e in g.get(&n).unwrap() {
			if !path.contains(e) && (!DIRESPECT || !e.directed || e.p1 == n) && pred.map_or(true, |f| f(e)) {
				let mut path = path.clone();
				path.push(e.clone());
				q.push((e.other(&n).clone(), path),  d + e.length);
			}
		}
	}
	None
}

fn graph_is_empty(g: &Graph) -> bool {
	g.values().all(Vec::is_empty)
}

fn graph_edges(g: &Graph) -> usize {
	g.values().map(|es| es.len()).sum::<usize>()/2
}

fn graph_find_edge(g: &Graph, p1: &NodeId, p2: &NodeId, discriminator: Option<&NodeId>) -> Option<Rc<Edge>> {
	g.get(p1).and_then(|es| es.iter().find(|e| e.other(p1) == p2 && e.discriminator.as_ref() == discriminator && e.iidx == 0)).map(Clone::clone)
}

fn graph_find_edges(g: &Graph, p1: &NodeId, p2: &NodeId) -> Vec<Rc<Edge>> {
	g.get(p1).map_or(vec![], |es| es.iter().filter_map(|e| if e.other(p1) == p2 { Some(e.clone()) } else { None }).collect())
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
fn bl33p<const DIRESPECT: bool>(mut g: Graph, sns: &Vec<NodeId>) -> Vec<Path> {
	let mut cycles: Vec<Path> = sns.iter().map(|_| vec![]).collect();
	let mut complete: Vec<bool> = sns.iter().map(|_| false).collect();
	while cycles.len() > 0 && !graph_is_empty(&g) {
		let i = (0..sns.len()).filter(|i| !complete[*i]).min_by_key(|i| path_length(&cycles[*i])).unwrap();
		let n = &sns[i];
		let cycle = &mut cycles[i];
		if let Some((v, y)) = if cycle.len() > 0 {
			let shmlop = path_shmlop(cycle, n);
			(0..shmlop.len()-1).filter_map(|i| {
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
			// log::trace!("inflating {} ({})", v, g.get(v).unwrap().len());
			let inj = bicycle::<DIRESPECT>(&g, v, None).unwrap();
			// log::trace!("with {}", inj.len());
			for e in &inj {
				e.remove(&mut g);
			}
			cycle.splice(y..y, inj);
		} else {
			complete[i] = true;
			if complete.iter().all(|b| *b) {
				g.retain(|_, es| es.len() > 0);
				log::warn!("Some sections of the graph are unreachable by along the road network from given starting points!: {:?}", g);
				break;
			}
		}
		log::trace!("{}|{} vs {}", g.len(), graph_edges(&g), cycles.iter().map(|c| format!("{}", c.len())).collect::<Vec<String>>().join("/"));
	}
	cycles
}

pub fn construct_flight_paths(roads: data::RoadGraph, drones: &data::Drones) -> Result<data::Paths, String> {
	let sns: Vec<NodeId> = drones.iter().flat_map(|l| roads.nodes.locate(l)).collect();
	if sns.len() < drones.len() {
		return Err("Failed to locate positions to the road graph".to_string());
	}
	log::info!("Located drones");
	let mut g = kreek::<false>(roads.try_into()?)?;
	log::info!("Kreeked road graph");
	let cycles = bl33p::<false>(g, &sns);
	log::info!("Bleeped cycles");
	Ok(cycles.into_iter().zip(sns.into_iter()).map(|(path, n0)| path_shmlop(&path, &n0).into_iter().map(|(node, discriminator)| data::PathSegment { node: node.clone(), discriminator: discriminator.map(Clone::clone) }).collect()).collect())
}

/// Merge snow samplings with following rules:
/// - between a sample without snow and a sample with some snow, sampling with snow wins
/// - depths of all samples for given road segment are averaged
pub fn merge_snow_statuses(snows: impl Iterator<Item = data::SnowStatusElement>) -> data::SnowStatuses {
	let mut keyed = IndexMap::new();
	for s in snows {
		let entry = keyed.entry((s.p1, s.p2, s.discriminator)).or_insert(f64s::ZERO);
		if *entry <= f64s::ZERO || s.depth <= f64s::ZERO {
			*entry = max(*entry, s.depth);
		} else {
			*entry = ((entry.f() + s.depth.f()) / 2.0).try_into().unwrap();
		}
	}
	keyed.into_iter().map(|((p1, p2, discriminator), depth)| data::SnowStatusElement { p1, p2, discriminator, depth }).collect()
}

pub mod meta {
	use crate::*;
	use serde::*;

	#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
	pub enum Recycle {
		/// do not move cycles
		No,
		/// move cycles between adjacent tours from expensive to cheap tour
		ExpensiveToCheap,
	}

	#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
	pub enum Clearing {
		/// the vehicle clears only the allocated edges
		OnlyAllocated,
		/// the vehicle clears all edges
		All,
	}
	
	#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
	pub enum Reorder {
		/// don't reorder
		No,
		/// swap 2 at random
		Swap2Random,
		/// generate new random order
		RandomReorder,
		/// swap most and least used
		Swap2MostLeast,
	}
	
	#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
	pub enum Realloc {
		/// don't
		No,
		/// swap 2 random links
		Swap2Random,
		/// move a link from vehicle that does most to one that does least
		MostToLeast,
	}

	#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
	pub struct Annealing {
		pub main_iterations: u64, //MI
		pub ft_iterations: u64, //II
		pub starting_temperature: f64, //ST
		pub cooling_factor: f64, //RC
	}

	#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
	pub struct Parameters {
		pub recycle: Recycle, //IV
		pub clearing: Clearing, //MD
		pub reorder: Reorder, //ChV
		pub realloc: Realloc, //MV
		pub annealing: Annealing,
		pub slowdown: f64s,
		pub weight_total: f64s,
		pub weight_max: f64s,
	}
}

