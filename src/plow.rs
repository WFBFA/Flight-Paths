use crate::*;
use graph::adapt::*;
use data::Distance;
use brr::meta::*;

use std::{collections::HashSet, convert::TryFrom};
use itertools::Itertools;
use rand::{Rng, prelude::SliceRandom};

type SID = u64;
type Coords = (f64, f64);

trait Positioned {
	fn pos(&self) -> Coords;
}

trait Weighted {
	fn weight(&self) -> f64s;
}

struct PlowSolver<N, E, Gen>
where
	N: IdentifiableNode + Positioned,
	E: graph::Edge<SID> + Weighted,
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
	E: graph::Edge<SID> + Weighted,
	Gen: Fn(&N::Id, SID) -> (SID, SID),
{
	/// allocates all snowy edges to some vehicle
	/// uses positions of vehicles as gravicenters of allocation clusters
	fn initial_allocation<'a>(&'a self, locs: &Vec<Coords>, snowy: impl Iterator<Item = &'a E>) -> Vec<HashSet<&'a E>> {
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
	/// updates allocation from solution
	fn sol_to_alloc<'a>(&'a self, order: impl Iterator<Item = usize>, sols: &Vec<Vec<&'a E>>, allocs: &mut Vec<HashSet<&'a E>>, snowy: impl Fn(&E) -> bool){
		for i in order {
			for e in &sols[i] {
				if snowy(e) {
					if allocs[i].insert(e) {
						for a in 0..allocs.len() {
							if a != i {
								allocs[a].remove(e);
							}
						}
					}
				}
			}
		}
	}
	/// iterative annealing solver
	fn solve<'a, const DIRESPECT: bool>(&'a self, sps: &Vec<SID>, locs: &Vec<Coords>, snowy: &HashSet<&'a E>, params: Parameters) -> Vec<Vec<&'a E>> {
		let vs = locs.len();
		let mut alloc = self.initial_allocation(locs, snowy.iter().map(|e| *e));
		let mut solution: Vec<Vec<&'a E>> = (0..vs).map(|_| Vec::new()).collect();
		log::debug!("Initialized allocations: {}", alloc.iter().map(|a| a.len()).join("/"));
		let mut rng = rand::thread_rng();
		let mut cost_max_best = f64s::INFINITY;
		let mut value_best = f64s::INFINITY;
		let mut temperature: f64 = params.annealing.starting_temperature;
		let mut ii = 0u64;
		let mut order: Vec<_> = (0..vs).collect();
		macro_rules! cycle_cost_compute {
			($sol:expr,$alloc:expr,$dun:expr) => {
				$sol.iter().map(|e| e.weight() * if snowy.contains(e) && if params.clearing == Clearing::All { !$dun.contains(e) } else { $alloc.contains(e) } { params.slowdown } else { f64s::try_from(1.0).unwrap() }).sum()
			};
			($sol:expr,$alloc:expr) => {
				$sol.iter().map(|e| e.weight() * if snowy.contains(e) && $alloc.contains(e) { params.slowdown } else { f64s::try_from(1.0).unwrap() }).sum()
			};
		}
		for _mi in 0..params.annealing.main_iterations {
			log::debug!("iteration {} current best {:.1}", _mi, value_best.f());
			//Try to improve allocations
			//TODO? change alloc
			//Shuffle evaluation order
			match params.reorder {
				Reorder::No => {},
				Reorder::Swap2Random => order.swap(rng.gen_range(0..vs), rng.gen_range(0..vs)),
				Reorder::Swap2MostLeast => {
					if let itertools::MinMaxResult::MinMax(i, j) = order.iter().cloned().minmax_by_key(|i| solution[*i].len()) {
						order.swap(i, j);
					}
				},
				Reorder::RandomReorder => order.shuffle(&mut rng),
			}
			log::debug!(" new order: {:?}", order);
			//Provide new solutions
			let mut sol_next: Vec<_> = (0..vs).map(|_| Vec::new()).collect();
			let mut cost_next_all = f64s::ZERO;
			let mut cost_next_max = f64s::ZERO;
			let mut costs_next = Vec::new();
			costs_next.resize(vs, f64s::ZERO);
			let mut dun = HashSet::new();
			for i in &order {
				let i = *i;
				log::debug!(" solving {}", i);
				match graph::heuristics::solve_pwrp::<_, _, _, _, _, DIRESPECT>(&self.graph.graph, sps[i], alloc[i].iter().map(|e| *e).filter(|e| !dun.contains(e)).collect(), |e| Some(e.weight())) {
					Ok(sol) => {
						let cost = cycle_cost_compute!(sol, alloc[i], dun);
						if params.clearing == Clearing::All {
							for e in &sol {
								dun.insert(*e);
							}
						}
						costs_next[i] = cost;
						cost_next_all = cost_next_all + cost;
						if cost > cost_next_max {
							cost_next_max = cost;
						}
						sol_next[i] = sol;
					}
					Err(_) => panic!("Can't reach everywhere :(") //TODO instead of panicking, try to reallocate unreachable sections first
				}
			}
			//Evaluate
			let sol_next = sol_next;
			let (cost_next_all, cost_next_max, costs_next) = (cost_next_all, cost_next_max, costs_next);
			let value_next = params.weight_total*cost_next_all + params.weight_max*cost_next_max;
			log::debug!(" new value: {:.5} costs: {}", value_next, costs_next.iter().join("|"));
			let sol_next = if value_next < value_best || (value_next <= value_best && cost_next_max < cost_max_best) {
				log::debug!(" solution accepted");
				solution = sol_next;
				value_best = value_next;
				cost_max_best = cost_next_max;
				if params.clearing == Clearing::All {
					self.sol_to_alloc(order.iter().cloned(), &solution, &mut alloc, |e| snowy.contains(e));
				}
				&solution
			} else {
				&sol_next
			};
			//Try to improve
			if params.recycle == Recycle::ExpensiveToCheap {
				let mut sol_improv = sol_next.clone();
				let mut vycles: Vec<Vec<_>> = sol_next.iter().zip(sps.iter()).map(|(path, n0)| graph::Graph::<SID, N, E>::path_to_nodes(path.iter().map(|e| *e), *n0).into_iter().map(|(v, _)| v).collect()).collect();
				for i in 0..vs {
					'nexc: for j in (i+1)..vs {
						let (i, j) = if costs_next[order[i]] > costs_next[order[j]] { (order[i], order[j]) } else { (order[j], order[i]) };
						for iu in 0..vycles[i].len() {
							for ju in 0..vycles[j].len() {
								if vycles[i][iu] == vycles[j][ju] {
									for iv in (iu+1)..vycles[i].len() {
										if vycles[i][iv] == vycles[i][iu] {
											// [i][iu..=iv] <=> [j][ju..=ju]
											// same as
											log::trace!("  [{}][{}..{}] => [{}][{}..{}]", i, iu, iv, j, ju, ju);
											let mine: Vec<_> = sol_improv[i].splice(iu..iv, vec![]).collect();
											sol_improv[j].splice(ju..ju, mine);
											let mine: Vec<_> = vycles[i].splice(iu..iv, vec![]).collect();
											vycles[j].splice(ju..ju, mine);
											//don't update costs to avoid swap-backs idk
											continue 'nexc;
										}
									}
								}
							}
						}
					}
				}
				//Evaluate improvements
				let sol_improv = sol_improv;
				let mut cost_improv_all = f64s::ZERO;
				let mut cost_improv_max = f64s::ZERO;
				let mut costs_improv = Vec::new();
				costs_improv.resize(vs, f64s::ZERO);
				for i in 0..vs {
					let cost = cycle_cost_compute!(sol_improv[i], alloc[i]);
					costs_improv[i] = cost;
					cost_improv_all = cost_improv_all + cost;
					if cost > cost_improv_max {
						cost_improv_max = cost;
					}
				}
				let (cost_improv_all, cost_improv_max, costs_improv) = (cost_improv_all, cost_improv_max, costs_improv);
				let value_improv = params.weight_total*cost_next_all + params.weight_max*cost_next_max;
				log::debug!(" new value: {:.5} costs: {}", value_improv, costs_improv.iter().join("|"));
				//if the improved solution is actually better, or with some chance anyway, keep it
				if value_improv < value_best || (value_improv <= value_best && cost_improv_max < cost_max_best) || (value_improv < value_next && rng.gen_range(0.0..1.0) < ((value_improv-value_next).f()/temperature).exp()) {
					log::debug!(" improvements accepted");
					solution = sol_improv;
					value_best = value_improv;
					cost_max_best = cost_improv_max;
					self.sol_to_alloc(order.iter().cloned(), &solution, &mut alloc, |e| snowy.contains(e));
				}
			}
			//Update the temperature
			ii += 1;
			if ii >= params.annealing.ft_iterations {
				ii = 0;
				temperature *= params.annealing.cooling_factor;
				log::debug!(" t={:.2}", temperature);
			}
		}
		solution
	}
}
