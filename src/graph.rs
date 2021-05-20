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
