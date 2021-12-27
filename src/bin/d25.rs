use std::io::{BufRead, BufReader};
use std::mem;

use eyre::{Result, eyre, bail};

fn main() -> Result<()> {
	let mut grid = BufReader::new(std::io::stdin())
		.lines()
		.map(|l| l.map(String::into_bytes).map_err(|e| eyre!(e)))
		.collect::<Result<Vec<Vec<u8>>>>()?;
	if grid.is_empty() {
		println!("0");
		return Ok(());
	}

	let width = grid[0].len();
	let height = grid.len();

	if grid.iter().skip(1).any(|l| l.len() != width) {
		bail!("non-rectangular input");
	}

	let mut steps = 0;
	let mut can_move = vec![vec![false; width]; height];

	loop {
		let mut any_moved = false;
		for line in &mut grid {
			for b in line {
				print!("{}", *b as char);
			}
			println!("");
		}
		println!("");

		for y in 0..height {
			for x in 0..width {
				can_move[y][x] = grid[y][x] == b'>' && grid[y][(x+1)%width] == b'.';
			}
		}

		for y in 0..height {
			for x in (0..width).rev() {
				if can_move[y][x] {
					any_moved = true;
					let nx = (x+1) % width;
					grid[y][x] = b'.';
					grid[y][nx] = b'>';
				}
			}
		}

		for y in 0..height {
			for x in 0..width {
				can_move[y][x] = grid[y][x] == b'v' && grid[(y+1)%height][x] == b'.';
			}
		}

		for y in (0..height).rev() {
			for x in 0..width {
				if can_move[y][x] {
					any_moved = true;
					let ny = (y+1) % height;
					grid[y][x] = b'.';
					grid[ny][x] = b'v';
				}
			}
		}

		steps += 1;

		if !any_moved {
			break;
		}
	}

	println!("{steps}");

	Ok(())
}
