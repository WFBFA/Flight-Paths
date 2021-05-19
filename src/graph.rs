use std::{collections::{HashMap, HashSet}, hash::Hash, rc::Rc};

pub trait Node<NId: Clone + Copy + Hash + Eq> : Clone {
	fn id(&self) -> NId;
}

pub trait Edge<NId: Clone + Copy + Hash + Eq> : Clone + Hash + PartialEq {
	type W: Clone + Copy + PartialEq + PartialOrd;
	fn p1(&self) -> NId;
	fn p2(&self) -> NId;
	fn directed(&self) -> bool;
	fn weight(&self) -> Self::W;
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
}
