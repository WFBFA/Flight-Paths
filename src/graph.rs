use std::{collections::{HashMap, HashSet}, hash::Hash, rc::Rc};

use indexmap::IndexSet;
use priority_queue::PriorityQueue;

pub trait Node<NId: Clone + Copy + Hash + Eq> : Clone {
	fn id(&self) -> NId;
}

pub trait Edge<NId: Clone + Copy + Hash + Eq> : Clone + Hash + PartialEq {
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

#[derive(Clone, Debug)]
pub struct Graph<NId, N, E> 
where 
	NId: Clone + Copy + Hash + Eq,
	N: Node<NId>,
	E: Edge<NId>,
{
	nodes: HashMap<NId, N>,
	edges: HashMap<NId, HashSet<Rc<E>>>,
}

impl<NId, N, E> Graph<NId, N, E>
where 
	NId: Clone + Copy + Hash + Eq,
	N: Node<NId>,
	E: Edge<NId>,
{
	pub fn new(nodes: HashMap<NId, N>, edges: HashMap<NId, HashSet<Rc<E>>>) -> Self {
		Self { nodes, edges }
	}
	pub fn empty() -> Self {
		Self {
			nodes: HashMap::new(),
			edges: HashMap::new(),
		}
	}
	pub fn get_node(&self, id: NId) -> Option<&N> {
		self.nodes.get(&id)
	}
	pub fn get_node_edges(&self, id: NId) -> Option<&HashSet<Rc<E>>> {
		self.edges.get(&id)
	}
	pub fn get_edges_between(&self, n1: NId, n2: NId) -> Vec<Rc<E>> {
		self.edges.get(&n1).iter().flat_map(|es| es.iter()).filter(|e| e.other(n1) == n2).cloned().collect()
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
	/// Calculate combined degree of a vertex
	pub fn degree<const DIRESPECT: bool>(&self, n: NId) -> isize {
		if let Some(es) = self.get_node_edges(n) {
			if DIRESPECT {
				-(es.iter().filter(|e| e.directed() && e.p1() == n).count() as isize - es.iter().filter(|e| e.directed() && e.p2() == n).count() as isize).abs() + es.iter().filter(|e| !e.directed()).count() as isize
			} else {
				es.len() as isize
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
	pub fn pathfind<Weight, const DIRESPECT: bool>(&self, n1: NId, n2: NId, weight: impl Fn(&Rc<E>) -> Option<Weight>) -> Option<Vec<Rc<E>>>
	where
		Weight: Clone + Copy + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>
	{
		let mut dp: HashMap<NId, (Weight, Option<Rc<E>>)> = HashMap::new();
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
			for e in self.get_node_edges(u).unwrap() {
				if !DIRESPECT || !e.directed() || e.p1() == u {
					if let Some(ed) = weight(e){
						let v = e.other(u);
						let d = d + ed;
						if dp.get(&v).map_or(true, |(vd, _)| vd > &d) {
							dp.insert(v.clone(), (d, Some(e.clone())));
							q.push(v.clone(), -d);
						}
					}
				}
			}
		}
		None
	}
	/// Find a cycle over vertex
	pub fn cycle_on<Weight, FW, const DIRESPECT: bool>(&self, n: NId, weight: FW) -> Option<Vec<Rc<E>>>
	where
		E: Eq,
		Weight: Clone + Copy + PartialEq + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&Rc<E>) -> Option<Weight>,
	{
		self.cycle_on_rec::<_, _, DIRESPECT>(n, n, Weight::default(), &mut IndexSet::new(), &weight)
	}
	fn cycle_on_rec<Weight, FW, const DIRESPECT: bool>(&self, n: NId, u: NId, d: Weight, steck: &mut IndexSet<Rc<E>>, weight: &FW) -> Option<Vec<Rc<E>>>
	where
		E: Eq,
		Weight: Clone + Copy + PartialEq + Ord + Default + std::ops::Add<Weight, Output = Weight> + std::ops::Neg<Output = Weight>,
		FW: Fn(&Rc<E>) -> Option<Weight>,
	{
		if u == n && d != Weight::default() {
			Some(steck.drain(..).rev().collect())
		} else {
			for e in self.get_node_edges(u).unwrap() {
				if !DIRESPECT || !e.directed() || e.p1() == u {
					if steck.insert(e.clone()) {
						if let Some(ed) = weight(e) {
							let v = e.other(u);
							let d = d + ed;
							if let Some(path) = self.cycle_on_rec::<_, _, DIRESPECT>(n, v, d, steck, weight) {
								return Some(path);
							}
						}
						steck.pop();
					}
				}
			}
			None
		}
	}
}
