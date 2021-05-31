use std::{cmp::max, collections::{HashMap, HashSet}, convert::{TryFrom, TryInto}, rc::Rc};

use indexmap::IndexMap;
use priority_queue::PriorityQueue;

use crate::*;

/// Merge snow samplings with following rules:
/// - between a sample without snow and a sample with some snow, sampling with snow wins
/// - depths of all samples for given road segment are averaged
pub fn merge_snow_statuses(snows: impl Iterator<Item = data::SnowStatusElement>) -> data::SnowStatuses {
	let mut keyed = IndexMap::new();
	for s in snows {
		let entry = keyed.entry((s.p1, s.p2, s.discriminator)).or_insert(n64(0.0));
		if *entry <= n64(0.0) || s.depth <= n64(0.0) {
			*entry = max(*entry, s.depth);
		} else {
			*entry = (*entry + s.depth) / n64(2.0);
		}
	}
	keyed.into_iter().map(|((p1, p2, discriminator), depth)| data::SnowStatusElement { p1, p2, discriminator, depth }).collect()
}

