use std::cell::Cell;
use std::io::{self, BufReader, BufRead};
use std::collections::BTreeSet;

use eyre::Result;

fn main() -> Result<()> {
	let mut risks: Vec<Vec<u32>> = Vec::new();
	for line in BufReader::new(io::stdin()).lines() {
		let line = line?;
		risks.push(line.bytes().map(|c| (c as u8 - b'0') as u32).collect());
	}

	let scale = 5;

	let sx = risks[0].len() * scale;
	let sy = risks.len() * scale;

	let state: Vec<Vec<(Cell<bool>, Cell<u32>)>> =
		vec![vec![(Cell::new(false), Cell::new(u32::MAX)); sx]; sy];

	let mut unvisited: BTreeSet<(&Cell<u32>, usize, usize)> = BTreeSet::new();
	for y in 0..sy {
		for x in 0..sx {
			unvisited.insert((&state[y][x].1, x as usize, y as usize));
		}
	}

	let (mut x, mut y) = (0usize, 0usize);
	state[y][x].1.set(0);

	let (target_x, target_y) = (sx-1, sy-1);

	loop {
		let c = &state[y][x];
		//println!("{}, {}, {}, {}", x, y, c.0.get(), c.1.get());

		if x == target_x && y == target_y {
			println!("{}", c.1.get());
			return Ok(());
		}

		for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
			let tx = (x as i32 + dx) as usize;
			let ty = (y as i32 + dy) as usize;
			if ty >= sy || tx >= sx { continue; }

			let n = &state[ty][tx];
			let adjustment = (ty / risks.len() + tx / risks[0].len()) as u32;
			let local_risk =
				((risks[ty % risks.len()][tx % risks[0].len()]
					+ adjustment) - 1) % 9 + 1;
			//println!("    {}, {}, {}, {}, {}", tx, ty, n.0.get(), n.1.get(), local_risk);
			if n.0.get() {
				continue;
			}

			let new_total_risk = c.1.get() + local_risk;
				//println!("      {} / {}", n.1.get(), new_total_risk);
			if new_total_risk < n.1.get() {
				let r = (&n.1, tx, ty);
				assert!(unvisited.remove(&r));
				n.1.set(new_total_risk);
				unvisited.insert(r);
			}
		}

		c.0.set(true);
		assert!(unvisited.remove(&(&c.1, x, y)));
		let n = unvisited.iter().next().unwrap();
		x = n.1;
		y = n.2;
	}
}
