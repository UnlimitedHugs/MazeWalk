use super::{distances::Distances, GridNode};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::slice::{ChunksExact, Iter, IterMut};

/// Swiped from https://github.com/strohs/maze-algorithms/rust

#[derive(Debug)]
pub struct GridMaze {
	nodes: Vec<GridNode>,
	links: HashMap<usize, Vec<usize>>,
	rows: usize,
	cols: usize,
}

impl GridMaze {
	/// constructs a new maze with the specified dimensions, using the `GridNode` type to represent
	/// each node of the maze. Nodes will be stored in row-order. Each node will have a default
	/// weight of 1 and its default value will be its one-dimensional index within the maze
	pub fn new(rows: usize, cols: usize) -> Self {
		let nodes = (0..(rows * cols))
			.into_iter()
			.map(|i| GridNode::new(i, 1))
			.collect();

		let links: HashMap<usize, Vec<usize>> = HashMap::new();

		Self {
			nodes,
			links,
			rows,
			cols,
		}
	}

	/// returns the dimensions of the maze as a (row, col) tuple
	pub fn dimensions(&self) -> (usize, usize) {
		(self.rows, self.cols)
	}

	/// returns the total number of nodes stored in this maze (i.e. rows * cols)
	pub fn len(&self) -> usize {
		self.rows * self.cols
	}

	/// returns a one-dimensional index based on the given row, col values
	/// `col_dim` is the number of columns in the maze
	pub fn idx_1d(row: usize, col: usize, col_dim: usize) -> usize {
		row * col_dim + col
	}

	/// returns the (col, row) position of a node with the given index in the maze
	pub fn idx_to_pos(&self, idx: usize) -> (i32, i32) {
		if idx >= self.len() {
			panic!("node index out of bounds");
		}
		((idx % self.cols) as i32, (idx / self.cols) as i32)
	}

	// /// generates a two dimensional index from a one-dimensional index. Returns it as a
	// /// `(row, col)` tuple
	// fn idx_2d(index: usize, col_dim: usize) -> (usize, usize) {
	//     (index / col_dim, index % col_dim)
	// }

	/// create a link between the two nodes in the maze. This will essentially create a passageway
	/// between them.
	/// `bi_link` creates a bi-directional link if it is `true`. Which means that in addition to
	/// creating a link from node1 => node2,  a link is also created from node2 => node1
	pub fn link(&mut self, node1: &GridNode, node2: &GridNode, bi_link: bool) {
		self.links
			.entry(node1.pos())
			.or_insert_with(|| vec![])
			.push(node2.pos());
		if bi_link {
			self.links
				.entry(node2.pos())
				.or_insert_with(|| vec![])
				.push(node1.pos());
		}
	}

	// returns copies of GridNode(s) that the given `node` links to.
	// In this particular maze, each node can have at most 4 links, or edges, to another Node
	// If the given node doesn't link to anything, an empty Vector is returned
	pub fn get_links(&self, node: &GridNode) -> Vec<GridNode> {
		match self.links.get(&node.pos()) {
			Some(linked_pos) => linked_pos.iter().map(|pos| self.nodes[*pos]).collect(),
			None => Vec::new(),
		}
	}

	/// returns `true` if there is a link between `node1` and `node2`, else `false`. Note this
	/// function only checks one-way links, it will not check for a link between `node2` and `node1`
	pub fn has_node_link(&self, node1: &GridNode, node2: &GridNode) -> bool {
		match self.links.get(&node1.pos()) {
			Some(node_links) => node_links.contains(&node2.pos()),
			None => false,
		}
	}

	/// returns true if the node has a neighbor in the given direction, as well as a link to it
	pub fn has_link(&self, node: &GridNode, direction: GridDirection) -> bool {
		if let Some(ref node2) = self.get_neighbor(node, direction) {
			return self.has_node_link(node, node2);
		}
		false
	}

	/// returns the neighbor of a node in the given direction, or None if node is at map edge
	pub fn get_neighbor(&self, node: &GridNode, direction: GridDirection) -> Option<GridNode> {
		match direction {
			GridDirection::Up => self.up(node),
			GridDirection::Right => self.right(node),
			GridDirection::Down => self.down(node),
			GridDirection::Left => self.left(node),
		}
	}

	/// returns the neighbors of the given `node`. Neighbors are the nodes adjacent to `node` but NOT
	/// necessarily linked to `node`. To get the linked nodes, use the `links()` function
	pub fn neighbors(&self, node: &GridNode) -> Vec<GridNode> {
		let neighbors = vec![
			self.up(node),
			self.right(node),
			self.down(node),
			self.left(node),
		];

		neighbors.into_iter().flatten().collect()
	}

	/// returns a copy of a random node in the maze
	pub fn random_node(&self) -> GridNode {
		let rand_idx = thread_rng().gen_range(0..self.nodes.len());
		self.nodes[rand_idx]
	}

	/// returns an immutable iterator over the *rows* of this maze
	pub fn iter_rows(&self) -> ChunksExact<'_, GridNode> {
		self.nodes.chunks_exact(self.cols)
	}

	/// returns an immutable iterator over this maze's Nodes in row order
	pub fn iter_nodes(&self) -> Iter<'_, GridNode> {
		self.nodes.iter()
	}

	/// returns a mutable iterator over this maze's nodes in row order
	pub fn iter_mut_nodes(&mut self) -> IterMut<'_, GridNode> {
		self.nodes.iter_mut()
	}

	/// returns a row or column of nodes adjacent to the edge of the maze in the given direction
	pub fn get_edge_nodes(&self, side: GridDirection) -> Vec<GridNode> {
		use GridDirection::*;
		match side {
			Up | Down => self
				.iter_rows()
				.nth(if side == Up { 0 } else { self.rows - 1 })
				.unwrap_or_default()
				.iter()
				.copied()
				.collect(),
			Right | Left => self
				.iter_rows()
				.filter_map(|r| if side == Right { r.last() } else { r.first() })
				.copied()
				.collect(),
		}
	}

	pub fn up(&self, node: &GridNode) -> Option<GridNode> {
		let node_index = node.pos();
		if node_index > self.cols {
			self.nodes.get(node_index - self.cols).copied()
		} else {
			None
		}
	}

	pub fn down(&self, node: &GridNode) -> Option<GridNode> {
		self.nodes.get(node.pos() + self.cols).copied()
	}

	pub fn right(&self, node: &GridNode) -> Option<GridNode> {
		// if the node is not at the right edge of the maze
		if node.pos() % self.cols + 1 != self.cols {
			self.nodes.get(node.pos() + 1).copied()
		} else {
			None
		}
	}

	pub fn left(&self, node: &GridNode) -> Option<GridNode> {
		// if node is not on the left edge of the maze
		if node.pos() % self.cols != 0 {
			self.nodes.get(node.pos() - 1).copied()
		} else {
			None
		}
	}
}

/// allows indexing into this maze using a single usize value that represents the one-dimensional
/// index of the Node you wish to retrieve
impl Index<usize> for GridMaze {
	type Output = GridNode;

	/// returns the Node at the specified index, `idx`, in this . The `idx` should be a
	/// one-dimensional index for the node to be retrieved
	fn index(&self, idx: usize) -> &Self::Output {
		&self.nodes[idx]
	}
}

/// pretty prints this Maze to standard out as ASCII characters
impl Display for GridMaze {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		// write the top wall of the maze
		writeln!(f, "+{}", "----+".repeat(self.cols))?;

		for row in self.iter_rows() {
			// top holds the node's 'bodies' (blank spaces) and right walls
			let mut top = String::from("|");
			// bottom holds the cell's southern wall and corners ('+') sign
			let mut bottom = String::from("+");

			for cur_node in row.iter() {
				// determine if a right wall should be drawn
				match self.right(cur_node) {
					Some(right_node) if self.has_node_link(&cur_node, &right_node) => {
						top.push_str("     ")
					}
					_ => top.push_str("    |"),
				}

				// determine if a southern wall should be drawn
				match self.down(cur_node) {
					Some(south_node) if self.has_node_link(&cur_node, &south_node) => {
						bottom.push_str("    +")
					}
					_ => bottom.push_str("----+"),
				}
			}

			writeln!(f, "{}", top)?;
			writeln!(f, "{}", bottom)?;
		}

		Ok(())
	}
}

/// Functions for converting a GridMaze into a Braided Maze
///
impl GridMaze {
	/// returns copies of the GridNodes in the Maze that are dead-ends. Dead-ends are Nodes that only
	/// have one link into/out-of them
	pub fn dead_ends(&self) -> Vec<GridNode> {
		self.links
			.iter()
			.filter(|(_pos, links)| links.len() == 1)
			.map(|(&pos, _links)| self.nodes[pos])
			.collect()
	}

	/// Adds braids to this maze by removing dead-end nodes and turning them into loops
	///
	/// `p` - is a value between 0.0 and 1.0 and is the percentage amount of dead-ends to remove.
	///       1.0 = remove all dead-ends, while a value of 0.5 would remove 50 percent of dead-ends
	pub fn braid(&mut self, p: f64) {
		// dead_ends is all the nodes in the Graph that are dead ends
		let mut dead_ends = self.dead_ends();
		dead_ends.shuffle(&mut thread_rng());

		for node in dead_ends {
			// make sure the position is still a dead-end, as it may have been changed in a
			// previous iteration of the loop
			if self.get_links(&node).len() != 1 || !thread_rng().gen_bool(p) {
				continue;
			} else {
				// now get neighbor nodes of `node` that are not linked to it
				let unlinked_neighbors = self
					.neighbors(&node)
					.into_iter()
					.filter(|neighbor| !self.has_node_link(&node, neighbor))
					.collect::<Vec<GridNode>>();

				// try to select a neighbors that are also dead end nodes
				let mut best_neighbors = unlinked_neighbors
					.iter()
					.filter(|&neighbor| self.get_links(neighbor).len() == 1)
					.copied()
					.collect::<Vec<GridNode>>();

				// if no best neighbors found, just use the unlinked neighbors
				if best_neighbors.is_empty() {
					best_neighbors = unlinked_neighbors;
				}

				// finally choose a random, best, neighbor and link to it
				if let Some(rand_neighbor) = best_neighbors.choose(&mut thread_rng()) {
					self.link(&node, rand_neighbor, true);
				}
			}
		}
	}
}

/// Functions to compute distances between nodes of a maze
impl GridMaze {
	/// find the distances from a `root` node to all other nodes in this `maze`, using each node's
	/// weight to compute the cost.
	/// returns a `Distances` struct containing the computed costs for each GridCell.
	pub fn distances(&self, root: &GridNode) -> Distances {
		// weights holds the Positions and current costs (weights) of the shortest path
		let mut weights = Distances::new(*root);

		// pending holds nodes that need to be visited
		let mut pending = vec![*root];

		while !pending.is_empty() {
			// sort pending so that cells with lowest weight are at the **end** of pending
			pending
				.sort_unstable_by(|&an, &bn| self[bn.pos()].weight().cmp(&self[an.pos()].weight()));

			// pop the last position from pending, it will have the lowest weight
			let cur_node = pending.pop().unwrap();

			// iterate thru the linked neighbors and compute the cost of moving into
			// each of them
			for neighbor_node in self.get_links(&cur_node) {
				// the total weight of moving into a neighboring node is the total weight
				// of the current path so far, plus the weight of the neighbor
				let total_weight =
					weights.get(&cur_node).unwrap() + self[neighbor_node.pos()].weight() as i32;

				// if the cost of moving into neighbor has not been recorded in the weights vector
				// OR the total cost of moving to neighbor is less than the current weight
				if weights.get(&neighbor_node).is_none()
					|| total_weight < *weights.get(&neighbor_node).unwrap()
				{
					pending.push(neighbor_node);
					weights.insert(neighbor_node, total_weight);
				}
			}
		}
		weights
	}

	/// pretty prints the `maze` and also displays each cell of `path` within its corresponding
	/// GridCell by printing its weight as a hexadecimal value.
	pub fn display_path(&self, path: &Distances) -> String {
		let mut buf = String::new();
		// write the top wall of the maze
		buf.push_str(&format!("+{} \n", "----+".repeat(self.cols)));

		for row in self.iter_rows() {
			// top holds the cell 'bodies' (blank spaces) and right walls
			let mut top = String::from("|");
			// bottom holds the cell's southern wall and corners ('+') sign
			let mut bottom = String::from("+");

			for cur_node in row.iter() {
				// if the current node is part of the path, we want to display the weight else a "  "
				let body = match path.get(cur_node) {
					Some(weight) => format!("{:3x}", weight),
					_ => String::from("   "),
				};

				// determine if a right wall should be drawn
				match self.right(cur_node) {
					Some(right_pos) if self.has_node_link(&cur_node, &right_pos) => {
						top.push_str(&format!("{}  ", body))
					}
					_ => top.push_str(&format!("{} |", body)),
				}

				// determine if a southern wall should be drawn
				match self.down(cur_node) {
					Some(south_pos) if self.has_node_link(&cur_node, &south_pos) => {
						bottom.push_str("    +")
					}
					_ => bottom.push_str("----+"),
				}
			}

			buf.push_str(&format!("{}\n", top));
			buf.push_str(&format!("{}\n", bottom));
		}
		buf
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum GridDirection {
	Up,
	Right,
	Down,
	Left,
}

impl GridDirection {
	pub const ALL: [GridDirection; 4] = [
		GridDirection::Up,
		GridDirection::Right,
		GridDirection::Down,
		GridDirection::Left,
	];

	pub fn get_offset(&self) -> (i32, i32) {
		match self {
			GridDirection::Up => (0, -1),
			GridDirection::Right => (1, 0),
			GridDirection::Down => (0, 1),
			GridDirection::Left => (-1, 0),
		}
	}
}

pub trait WorldDirections {
	fn north(&self, node: &GridNode) -> Option<GridNode>;
	fn south(&self, node: &GridNode) -> Option<GridNode>;
	fn east(&self, node: &GridNode) -> Option<GridNode>;
	fn west(&self, node: &GridNode) -> Option<GridNode>;
}
impl WorldDirections for GridMaze {
	fn north(&self, node: &GridNode) -> Option<GridNode> {
		self.up(node)
	}
	fn south(&self, node: &GridNode) -> Option<GridNode> {
		self.down(node)
	}
	fn east(&self, node: &GridNode) -> Option<GridNode> {
		self.right(node)
	}
	fn west(&self, node: &GridNode) -> Option<GridNode> {
		self.left(node)
	}
}

#[cfg(test)]
mod tests {
	use super::GridMaze;

	#[test]
	fn create_new_maze_with_9_nodes() {
		let maze = GridMaze::new(3, 3);
		assert_eq!(maze.nodes.len(), 9);
	}

	#[test]
	fn should_create_new_maze_with_all_nodes_of_weight_1() {
		let maze = GridMaze::new(3, 3);
		for node in &maze.nodes {
			assert_eq!(node.weight(), 1);
		}
	}

	#[test]
	fn maze_has_3x4_dimension() {
		let maze = GridMaze::new(3, 4);
		let (r, c) = maze.dimensions();
		assert_eq!(r, 3);
		assert_eq!(c, 4);
	}

	#[test]
	fn should_bi_link_two_nodes() {
		let mut maze = GridMaze::new(3, 3);
		let n1 = maze[0];
		let n2 = maze[1];
		maze.link(&n1, &n2, true);
		assert!(maze.links.contains_key(&n1.pos()));
		assert!(maze.links.get(&n1.pos()).unwrap().contains(&n2.pos()));
		assert!(maze.links.contains_key(&n2.pos()));
		assert!(maze.links.get(&n2.pos()).unwrap().contains(&n1.pos()));
	}

	#[test]
	fn should_get_links() {
		let mut maze = GridMaze::new(3, 3);
		let n00 = maze[0];
		let n01 = maze[1];
		let n10 = maze[3];
		maze.link(&n00, &n01, true);
		maze.link(&n00, &n10, true);
		let n0_links = maze.get_links(&n00);
		assert_eq!(n0_links.len(), 2);
	}

	#[test]
	fn should_index_into_graph() {
		let maze = GridMaze::new(3, 3);
		// get the node at row 1, column 1, who's one-dimensional index should = 4;
		let node11 = maze[4];
		assert_eq!(node11.pos(), 4);
	}

	#[test]
	fn node_0_should_not_have_north_neighbor() {
		let maze = GridMaze::new(3, 3);
		let node0 = maze[0];
		assert_eq!(maze.up(&node0), None);
	}

	#[test]
	fn node_0_should_not_have_left_neighbor() {
		let maze = GridMaze::new(3, 3);
		let node0 = maze[0];
		assert_eq!(maze.left(&node0), None);
	}

	#[test]
	fn node_6_should_have_north_neighbor() {
		let maze = GridMaze::new(3, 3);
		let node6 = maze[6];
		assert_eq!(maze.up(&node6), Some(maze[3]));
	}

	#[test]
	fn node_2_should_not_have_right_neighbor() {
		let maze = GridMaze::new(3, 3);
		let node = maze[2];
		assert_eq!(maze.right(&node), None);
	}

	#[test]
	fn node_3_should_have_right_neighbor() {
		let maze = GridMaze::new(3, 3);
		let node = maze[3];
		assert_eq!(maze.right(&node), Some(maze[4]));
	}

	#[test]
	fn node_8_should_not_have_right_neighbor() {
		let maze = GridMaze::new(3, 3);
		let node = maze[8];
		assert_eq!(maze.right(&node), None);
	}

	#[test]
	fn node_4_should_have_all_neighbors() {
		let maze = GridMaze::new(3, 3);
		let node = maze[4];
		assert_eq!(maze.right(&node), Some(maze[5]));
		assert_eq!(maze.up(&node), Some(maze[1]));
		assert_eq!(maze.left(&node), Some(maze[3]));
		assert_eq!(maze.down(&node), Some(maze[7]));
	}

	#[test]
	fn should_display_maze_with_link_from_00_to_01() {
		let mut maze = GridMaze::new(4, 4);
		let n00 = maze[0];
		let n01 = maze[1];
		maze.link(&n00, &n01, true);
		println!("{}", &maze);
	}

	#[test]
	fn directions_map_to_correct_nodes() {
		use super::GridDirection::*;
		let maze = GridMaze::new(3, 3);
		let center = &maze[4];
		let dirs = [Up, Right, Down, Left]
			.iter()
			.map(|d| maze.get_neighbor(center, *d));
		let calls = [
			maze.up(center),
			maze.right(center),
			maze.down(center),
			maze.left(center),
		];
		for (a, b) in dirs.zip(calls.iter()) {
			assert_eq!(a, *b);
		}
	}

	#[test]
	fn get_edge_nodes() {
		use super::GridDirection::*;
		let maze1 = GridMaze::new(3, 3);
		assert_eq!(maze1.get_edge_nodes(Up), vec![maze1[0], maze1[1], maze1[2]]);
		assert_eq!(
			maze1.get_edge_nodes(Down),
			vec![maze1[6], maze1[7], maze1[8]]
		);
		assert_eq!(
			maze1.get_edge_nodes(Right),
			vec![maze1[2], maze1[5], maze1[8]]
		);
		assert_eq!(
			maze1.get_edge_nodes(Left),
			vec![maze1[0], maze1[3], maze1[6]]
		);

		let maze2 = GridMaze::new(1, 1);
		let maze2_single = vec![maze2[0]];
		assert_eq!(maze2.get_edge_nodes(Up), maze2_single);
		assert_eq!(maze2.get_edge_nodes(Down), maze2_single);
		assert_eq!(maze2.get_edge_nodes(Right), maze2_single);
		assert_eq!(maze2.get_edge_nodes(Left), maze2_single);
	}

	#[test]
	fn index_to_position_conversion() {
		let maze = GridMaze::new(3, 3);
		assert_eq!(maze.idx_to_pos(0), (0, 0));
		assert_eq!(maze.idx_to_pos(2), (2, 0));
		assert_eq!(maze.idx_to_pos(4), (1, 1));
		assert_eq!(maze.idx_to_pos(8), (2, 2));
	}

	#[test]
	#[should_panic]
	fn invalid_index_to_position() {
		let maze = GridMaze::new(3, 3);
		maze.idx_to_pos(9);
	}
}
