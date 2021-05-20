use std::{collections::{HashMap, HashSet}, hash::Hash};

use indexmap::IndexMap;
use priority_queue::PriorityQueue;

pub trait Edge<NId: Clone + Copy + Hash + Eq> : Clone + Hash + PartialEq + Eq {
	fn p1(&self) -> NId;
	fn p2(&self) -> NId;
	fn directed(&self) -> bool;
	fn is_cyclic(&self) -> bool {
		self.p1() == self.p2()
	}
	fn other(&self, id: NId) -> NId {
		if id == self.p1() {
			self.p2()
		} else {
			self.p1()
		}
	}
	fn is_incoming<const DIRESPECT: bool>(&self, id: NId) -> bool {
		id == self.p2() || (id == self.p1() && (!DIRESPECT || !self.directed()))
	}
	fn is_outgoing<const DIRESPECT: bool>(&self, id: NId) -> bool {
		id == self.p1() || (id == self.p2() && (!DIRESPECT || !self.directed()))
	}
}

#[derive(Clone, Default, Debug)]
pub struct Graph<NId, N, E> 
where 
	NId: Clone + Copy + Hash + Eq,
	E: Edge<NId>,
{
	nodes: HashMap<NId, N>,
	edges: IndexMap<NId, HashSet<E>>,
}

impl<NId, N, E> Graph<NId, N, E>
where 
	NId: Clone + Copy + Hash + Eq,
	E: Edge<NId>,
{
	pub fn new(nodes: HashMap<NId, N>, edges: IndexMap<NId, HashSet<E>>) -> Self {
		Self { nodes, edges }
	}
	pub fn get_node(&self, n: NId) -> Option<&N> {
		self.nodes.get(&n)
	}
	pub fn get_edges(&self, n: NId) -> Option<&HashSet<E>> {
		self.edges.get(&n)
	}
	pub fn get_edges_between(&self, n1: NId, n2: NId) -> Vec<&E> {
		self.edges.get(&n1).iter().flat_map(|es| es.iter()).filter(|e| e.other(n1) == n2).collect()
	}
	pub fn node_count(&self) -> usize {
		self.nodes.len()
	}
	pub fn edge_count(&self) -> usize {
		self.edges.values().map(HashSet::len).sum::<usize>()/2
	}
	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}
	pub fn is_edge_empty(&self) -> bool {
		self.edges.values().all(HashSet::is_empty)
	}
	/// Adds (or replaces) a node
	pub fn add_node(&mut self, id: NId, n: N) -> Option<N> {
		self.edges.entry(id).or_default();
		self.nodes.insert(id, n)
	}
	/// Adds an edge
	pub fn add_edge(&mut self, e: E) -> bool {
		if self.nodes.contains_key(&e.p1()) && self.nodes.contains_key(&e.p2()) {
			if !e.is_cyclic() {
				self.edges.entry(e.p1()).or_default().insert(e.clone());
			}
			self.edges.entry(e.p2()).or_default().insert(e);
			true
		} else {
			false
		}
	}
	/// Calculate combined degree of a vertex
	pub fn degree<const DIRESPECT: bool>(&self, n: NId) -> isize {
		if let Some(es) = self.get_edges(n) {
			if DIRESPECT {
				-(es.iter().filter(|e| e.directed() && e.p1() == n).count() as isize - es.iter().filter(|e| e.directed() && e.p2() == n).count() as isize).abs() + es.iter().filter(|e| !e.directed() && !e.is_cyclic()).count() as isize
			} else {
				es.iter().filter(|e| !e.is_cyclic()).count() as isize
			}
		} else {
			0
		}
	}
	/// Check whether given node does not prevent a graph from being eulirian
	pub fn eulirian_compatible<const DIRESPECT: bool>(&self, n: NId) -> bool {
		let d = self.degree::<DIRESPECT>(n);
		d % 2 == 0 && (!DIRESPECT || d >= 0)
	}
	/// Find shortest path between 2 points
	pub fn pathfind<Weight, FW, const DIRESPECT: bool>(&self, n1: NId, n2: NId, weight: FW) -> Option<Vec<&E>>
	where
		Weight: Clone + Copy + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&E) -> Option<Weight>,
	{
		let mut dp: HashMap<NId, (Weight, Option<&E>)> = HashMap::new();
		dp.insert(n1.clone(), (Weight::default(), None));
		let mut q = PriorityQueue::new();
		q.push(n1.clone(), Weight::default());
		while let Some((u, _)) = q.pop() {
			if u == n2 {
				let mut path = Vec::new();
				let mut v = u;
				while let Some((_, Some(e))) = dp.get(&v) {
					v = e.other(v);
					path.push(e.clone());
				}
				path.reverse();
				return Some(path);
			}
			let d = dp.get(&u).unwrap().0;
			for e in self.get_edges(u).unwrap() {
				if !DIRESPECT || !e.directed() || e.p1() == u {
					if let Some(ed) = weight(e){
						let v = e.other(u);
						let d = d + ed;
						if dp.get(&v).map_or(true, |(vd, _)| vd > &d) {
							dp.insert(v.clone(), (d, Some(e)));
							q.push(v.clone(), -d);
						}
					}
				}
			}
		}
		None
	}
	/// Find shortest path between 2 regions
	pub fn pathfind_regions<Weight, FW, const DIRESPECT: bool>(&self, n1: &HashSet<NId>, n2: &HashSet<NId>, weight: FW) -> Option<(NId, NId, Vec<&E>)>
	where
		Weight: Clone + Copy + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&E) -> Option<Weight>,
	{
		if n1.is_empty() || n2.is_empty() {
			return None;
		}
		let mut dp: HashMap<NId, (Weight, Option<&E>)> = HashMap::new();
		let mut q = PriorityQueue::new();
		for n1 in n1 {
			dp.insert(n1.clone(), (Weight::default(), None));
			q.push(n1.clone(), Weight::default());
		}
		while let Some((u, _)) = q.pop() {
			if n2.contains(&u) {
				let mut path = Vec::new();
				let mut v = u;
				while let Some((_, Some(e))) = dp.get(&v) {
					v = e.other(v);
					path.push(e.clone());
				}
				path.reverse();
				return Some((v, u, path));
			}
			let d = dp.get(&u).unwrap().0;
			for e in self.get_edges(u).unwrap() {
				if !DIRESPECT || !e.directed() || e.p1() == u {
					if let Some(ed) = weight(e){
						let v = e.other(u);
						let d = d + ed;
						if dp.get(&v).map_or(true, |(vd, _)| vd > &d) {
							dp.insert(v.clone(), (d, Some(e)));
							q.push(v.clone(), -d);
						}
					}
				}
			}
		}
		None
	}
	/// Find a cycle over vertex
	pub fn cycle_on<Weight, FW, const DIRESPECT: bool>(&self, n: NId, weight: FW) -> Option<Vec<&E>>
	where
		E: Eq,
		Weight: Clone + Copy + PartialEq + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&E) -> Option<Weight>,
	{
		let mut q = PriorityQueue::new();
		q.push((n, Vec::new()), Weight::default()); //FIXME can't use IndexSet coz it doesn't impl Hash :(
		while let Some(((u, path), d)) = q.pop() {
			if u == n && !path.is_empty() {
				return Some(path);
			}
			for e in self.get_edges(u).unwrap() {
				if !path.contains(&e) && (!DIRESPECT || !e.directed() || e.p1() == u) {
					if let Some(ed) = weight(e) {
						let mut path = path.clone();
						path.push(e);
						q.push((u, path), d + ed); //FIXME i don't know why this works, but it does
					}
				}
			}
		}
		None
	}
	/// Make the graph eulirian by duplicating edges.
	///
	/// The only possible reason for failure is when there is a directed edge going nowhere, in which case the offending edge is reurned.
	pub fn eulirianize<P, FS, FP, FD, const DIRESPECT: bool>(&mut self, duped: FS, priority: FP, dupe: FD) -> Result<(), &E>
	where
		P: Ord,
		FS: Fn(&E, &E) -> bool,
		FP: Fn(&E) -> Option<P>,
		FD: Fn(&E) -> E,
	{
		for i in 0..self.edges.len() {
			let (u, es) = self.edges.get_index(i).unwrap();
			if es.len() == 1 {
				let e = es.iter().next().unwrap();
				if DIRESPECT && e.directed() && e.p2() == *u {
					return Err(self.edges.get_index(i).unwrap().1.iter().next().unwrap());
				}
				self.add_edge(dupe(e));
			}
		}
		while let Some((u, es)) = self.edges.iter().find(|(u, _)| !self.eulirian_compatible::<DIRESPECT>(**u)) {
			let u = *u;
			let epre = es.iter().filter(|e| !e.is_cyclic() && !es.iter().any(|ee| duped(e, ee)) && priority(e).is_some());
			let mut es: Vec<_> = if DIRESPECT {
				let ind = es.iter().filter(|e| e.directed() && e.p2() == u).count();
				let outd = es.iter().filter(|e| e.directed() && e.p1() == u).count();
				epre.filter(|e| !e.directed() || (outd > ind && e.p2() == u) || (ind > outd && e.p1() == u)).collect()
			} else {
				epre.collect()
			};
			es.sort_unstable_by_key(|e| priority(e));
			self.add_edge(dupe(es[0]));
		}
		Ok(())
	}
	/// Converts a path consisting of successive edges to successively visited nodes
	pub fn path_to_nodes<'a>( path: impl Iterator<Item = &'a E>, n: NId) -> Vec<(NId, Option<&'a E>)> { //TODO this can become a generator one day!
		let mut vs = vec![(n, None)];
		for e in path {
			vs.push((e.other(vs.last().unwrap().0), Some(e)));
		}
		vs
	}
}

pub mod heuristics {
	use super::*;
	
	/// Solve Positioned Windy Rural Postman
	pub fn solve_pwrp<'a, NId, N, E, Weight, FW, const DIRESPECT: bool>(g: &'a Graph<NId, N, E>, sp: NId, mut alloc: HashSet<&'a E>, weight: FW) -> Result<Vec<&'a E>, HashSet<&'a E>>
	where 
		NId: Clone + Copy + Hash + Eq,
		E: Edge<NId>,
		Weight: Clone + Copy + PartialEq + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&E) -> Option<Weight>,
	{
		log::trace!("Solving PWRP, starting with {}", alloc.len());
		let mut sol: Vec<&E> = Vec::new();
		macro_rules! sol_weight {
			() => {
				|e| if !sol.contains(&e) { weight(e) } else { None }
			}
		}
		macro_rules! sol_inject {
			($inj:expr,$y:expr) => {
				log::trace!("of {}", $inj.len());
				for e in &$inj {
					alloc.remove(e);
				}
				log::trace!("remaining {}", alloc.len());
				sol.splice($y..$y, $inj);
			}
		}
		while !alloc.is_empty() {
			if let Some((v, y)) = Graph::<NId, N, E>::path_to_nodes(sol.iter().map(|e| *e), sp).into_iter().enumerate().find_map(|(i, (v, _))| if g.get_edges(v).unwrap().iter().any(|e| !sol.contains(&e) && alloc.contains(e)) { Some((v, i)) } else { None }) {
				log::trace!("injecting a cycle");
				let inj = g.cycle_on::<_, _, DIRESPECT>(v, sol_weight!()).unwrap();
				sol_inject!(inj, y);
			} else {
				log::trace!("connecting to a distant isle");
				let vs: HashSet<_> = alloc.iter().flat_map(|e| if !DIRESPECT || !e.directed() { vec![e.p1(), e.p2()] } else { vec![e.p1()] }).collect();
				let us: IndexMap<_, _> = Graph::<NId, N, E>::path_to_nodes(sol.iter().map(|e| *e), sp).into_iter().enumerate().filter_map(|(i, (u, _))| {
					let ures: Vec<_> = g.get_edges(u).unwrap().iter().filter(|e| !sol.contains(e)).collect();
					if ures.iter().filter(|e| e.is_incoming::<DIRESPECT>(u)).count() > 0 && ures.iter().filter(|e| e.is_outgoing::<DIRESPECT>(u)).count() > 0 {
						Some((u, i))
					} else {
						None
					}
				}).collect();
				if let Some((inj, y)) = if let Some((u, v, mut p)) = g.pathfind_regions::<_, _, DIRESPECT>(&us.keys().cloned().collect(), &vs, sol_weight!()) {
					let e = alloc.iter().find(|e| e.is_outgoing::<DIRESPECT>(v)).unwrap();
					p.push(*e);
					if let Some(mut pb) = g.pathfind::<_, _, DIRESPECT>(e.other(v), u, |e| if !sol.contains(&e) && !p.contains(&e) { weight(e) } else { None }) {
						p.append(&mut pb);
						// log::trace!("connecting {} to {} to {} to {}", u, v, e.other(v), u);
						Some((p, *us.get(&u).unwrap()))
					} else {
						None
					}
				} else {
					None
				} {
					sol_inject!(inj, y);
				} else {
					log::trace!("failed to reach");
					return Err(alloc);
				}
			}
		}
		log::trace!("solved visiting {} segments", sol.len());
		Ok(sol)
	}
}
