use super::{grid_maze::WorldDirections, GridMaze, GridNode};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

/// Generates a random maze using the Sidewinder algorithm. It's similar to binary tree but
/// does have some differences. In a nutshell, it goes like this:
///  1. Work through the maze row-wise, starting with the cell at 0,0. Initialize the “run” set to be empty.
///  2. Add the current cell to the “run” set.
///  3. For the current cell, randomly decide whether to carve east or not.
///  4. If a passage was carved, make the new cell the current cell and repeat steps 2-4.
///  5. If a passage was not carved, choose any one of the cells in the run set and carve a
///     passage north. Then empty the run set, set the next cell in the row to be the current
///     cell, and repeat steps 2-5.
///  6. Continue until all rows have been processed.
pub fn generate(height: usize, width: usize) -> GridMaze {
	let at_eastern_boundary = |maze: &GridMaze, cell: &GridNode| maze.east(cell).is_none();
	let at_northern_boundary = |maze: &GridMaze, cell: &GridNode| maze.north(cell).is_none();
	// should we close out the current run of cells
	let should_close_out = |maze: &GridMaze, cell: &GridNode| {
		at_eastern_boundary(maze, cell)
			|| (!at_northern_boundary(maze, cell) && thread_rng().gen::<bool>())
	};

	let mut maze = GridMaze::new(height, width);

	for cur_index in 0..maze.len() {
		let cur_node = maze[cur_index];
		let mut runs = vec![cur_node];

		if should_close_out(&maze, &cur_node) {
			let rand_member = runs.choose(&mut thread_rng());

			// if the random_member has a north neighbor, carve a passage from the random cell
			// to it's north neighbor
			if let Some(rand_node) = rand_member {
				if let Some(north_pos) = maze.north(rand_node) {
					maze.link(&rand_node, &north_pos, true);
				}
			}
			runs.clear();
		} else {
			// carve a passage from current cell to the east neighbor
			if let Some(east_node) = maze.east(&cur_node) {
				maze.link(&cur_node, &east_node, true);
			}
		}
	}

	maze
}
