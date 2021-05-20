use crate::*;
use graph::adapt::*;
use data::Distance;

use std::{collections::HashSet, convert::TryFrom};

type SID = u64;
type Coords = (f64, f64);

trait Positioned {
	fn pos(&self) -> Coords;
}

struct PlowSolver<N, E, Gen>
where
	N: IdentifiableNode + Positioned,
	E: graph::Edge<SID>,
	Gen: Fn(&N::Id, SID) -> (SID, SID),
{
	graph: GraphAdapter<SID, N, E, SID, Gen>,
}
macro_rules! plow_solver {
	() => {
		PlowSolver {
			graph: GraphAdapter::new(0, |_, id| (id, id+1)),
		}
	}
}

impl<N, E, Gen> PlowSolver<N, E, Gen>
where
	N: IdentifiableNode + Positioned,
	E: graph::Edge<SID>,
	Gen: Fn(&N::Id, SID) -> (SID, SID),
{
	fn initial_allocation<'a>(&'a self, locs: Vec<Coords>, snowy: impl Iterator<Item = &'a E>) -> Vec<HashSet<&'a E>> {
		let closest = |c: &(f64, f64)| (0..locs.len()).zip(locs.iter()).min_by_key(|(_, c2)| f64s::try_from(c.distance(*c2)).unwrap()).unwrap().0;
		let mut allocations: Vec<_> = (0..locs.len()).map(|_| HashSet::new()).collect();
		for e in snowy {
			let lv1 = closest(&self.graph.nid2node(e.p1()).unwrap().pos());
			let lv2 = closest(&self.graph.nid2node(e.p2()).unwrap().pos());
			let lv = if lv1 == lv2 || allocations[lv2].len() > allocations[lv1].len() { lv1 } else { lv2 };
			allocations[lv].insert(e);
		}
		allocations
	}
}
