use std::hash::{Hash, Hasher};

/// GridNode is the default type that can be stored in a maze maze. GridNodes contain a `idx` index
/// field that is used to uniquely identify a node's index in a maze grid.  Additionally. maze
/// nodes have a `weight` field that can be used to store cost calculations for maze solvers etc..
#[derive(Debug, Copy, Clone)]
pub struct GridNode {
	idx: usize,
	weight: isize,
}

impl GridNode {
	// constructs a new Node with the specified `idx` and `weight`
	pub fn new(idx: usize, weight: isize) -> Self {
		GridNode { idx, weight }
	}

	// returns the idx of this node
	pub fn idx(&self) -> usize {
		self.idx
	}

	// returns the weight of the node
	pub fn weight(&self) -> isize {
		self.weight
	}

	pub fn set_idx(&mut self, new_idx: usize) {
		self.idx = new_idx;
	}

	pub fn set_weight(&mut self, new_weight: isize) {
		self.weight = new_weight;
	}
}

impl PartialEq for GridNode {
	/// two maze nodes are considered equal if their respective `idx` are equal
	fn eq(&self, other: &Self) -> bool {
		self.idx == other.idx
	}
}

impl Eq for GridNode {}

// HASH impl
impl Hash for GridNode {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.idx.hash(state);
	}
}

#[cfg(test)]
mod tests {
	use super::GridNode;
	use std::collections::hash_map::DefaultHasher;
	use std::hash::{Hash, Hasher};

	#[test]
	fn should_create_new_node() {
		let node = GridNode::new(1, 125);
		assert_eq!(node.idx, 1);
		assert_eq!(node.weight, 125);
	}

	#[test]
	fn equal_idx_should_hash_to_equal_hashes() {
		let mut hasher = DefaultHasher::new();
		let node1 = GridNode::new(1, 111);
		let node2 = GridNode::new(1, 222);
		node1.hash(&mut hasher);

		let mut hasher2 = DefaultHasher::new();
		node2.hash(&mut hasher2);
		assert_eq!(hasher.finish(), hasher2.finish());
	}
}
