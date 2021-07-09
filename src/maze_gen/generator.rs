use super::{GridMaze, GridNode};
use rand::{thread_rng, seq::SliceRandom};

/// Generates a random maze using Wilson's algorithm:
/// Like Aldous-Broder, this algorithm depends on the idea of a random walk, but with a twist.
/// It performs what is called a loop-erased random walk, which means that as it goes, if the path
/// it is forming happens to intersect with itself and form a loop, it erases that loop before
/// continuing on.
///
/// 1. choose a point on the maze and mark it visited.
/// 2. choose any unvisited node in the maze and perform a loop-erased random walk until you
///    reach a visited node.
/// 3. link all the nodes in the current random walk to the visited node
/// 4. repeat step 2 until all nodes in the maze have been visited
pub fn generate(height: usize, width: usize) -> GridMaze {
    let mut maze = GridMaze::new(height, width);

    // choose a random node in the maze, this will be the first visited node
    let first = maze.random_node();
    // initialize unvisited to contain all positions in the maze except for first
    let mut unvisited_nodes: Vec<GridNode> = maze
        .iter_nodes()
        .filter(|&node| *node != first)
        .copied()
        .collect();

    // repeat until all nodes have been visited
    while !unvisited_nodes.is_empty() {
        // choose a random, unvisited node and add it to the `path` that is about to be walked
        let mut cur_node = *unvisited_nodes.choose(&mut thread_rng()).unwrap();
        // path contains the randomly walked nodes
        let mut path: Vec<GridNode> = vec![cur_node];

        // while the cur_node is a member of unvisited nodes
        while unvisited_nodes.contains(&cur_node) {
            // choose a random neighbor of the current node
            cur_node = *maze
                .neighbors(&cur_node)
                .choose(&mut thread_rng())
                .expect("all nodes will have at least two neighbors");

            // if the random neighbor is already in path, there is a loop, so remove it
            if let Some(node_index) = path.iter().position(|node| *node == cur_node) {
                path = path[0..=node_index].to_vec();
            } else {
                // the random neigbor is not going to make a loop, so push it onto the path
                path.push(cur_node);
            }
        }

        // carve passages (i.e. link) between the nodes in path
        let mut window = path.windows(2);
        while let Some([node1, node2]) = window.next() {
            maze.link(node1, node2, true);

            // remove the nodes in the path from the vector of unvisited nodes
            if let Some(path_idx) = unvisited_nodes.iter().position(|node| *node == *node1) {
                unvisited_nodes.remove(path_idx);
            }
        }
    }

    maze
}
