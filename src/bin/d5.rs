use std::io::{self, BufReader, BufRead};
use std::str;

use eyre::{Result, eyre, bail};

#[derive(Debug, Clone, Copy)]
struct Pos(usize, usize);

impl str::FromStr for Pos {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Pos> {
		let mut iter = s.split(',');
		match (iter.next(), iter.next(), iter.next()) {
			(Some(x), Some(y), None) => Ok(Pos(x.parse()?, y.parse()?)),
			_ => bail!("expected two coords"),
		}
	}
}

fn main() -> Result<()> {
	let lines: Vec<_> = BufReader::new(io::stdin()).lines()
		.collect::<Result<_, _>>()?;
	let mut lines: Vec<(Pos, Pos)> = lines.into_iter().map(|line| {
		let mut points = line.split(" -> ").map(|coords| coords.parse());
		match (points.next(), points.next(), points.next()) {
			(Some(p), Some(q), None) => Ok((p?, q?)),
			_ => bail!("expected two points"),
		}
	}).collect::<Result<_, _>>()?;

	//dbg!(&lines);

	let mut grid = Vec::new();

	for (p, q) in lines {
		draw_line(&mut grid, p, q);
		//eprintln!("{:?} -> {:?}", p, q);
	}

	//for line in &grid {
	//	for &cell in line {
	//		print!("{}", match cell {
	//			0 => ' ',
	//			_ if cell > 9 => '#',
	//			_ => (b'0' + cell as u8) as char,
	//		});
	//	}
	//	println!("");
	//}

	let count = count_intersections(&grid, 2);
	println!("{count}");

	Ok(())
}

fn ensure_fits(grid: &mut Vec<Vec<usize>>, Pos(x, y): Pos) {
	if grid.len() <= y {
		grid.resize_with(y + 1, Vec::new);
	}

	if grid[y].len() <= x {
		grid[y].resize(x + 1, 0);
	}
}

fn draw_line(grid: &mut Vec<Vec<usize>>, Pos(x1, y1): Pos, Pos(x2, y2): Pos) {
	if x1 == x2 {
		let (y1, y2) = (usize::min(y1, y2), usize::max(y1, y2));
		for y in y1..=y2 {
			ensure_fits(grid, Pos(x1, y));
			grid[y][x1] += 1;
		}
	} else if y1 == y2 {
		let (x1, x2) = (usize::min(x1, x2), usize::max(x1, x2));
		for x in x1..=x2 {
			ensure_fits(grid, Pos(x, y1));
			grid[y1][x] += 1;
		}
	} else if abs_diff(x1, x2) == abs_diff(y1, y2) {
		let dx = if x1 > x2 { !0 } else { 1 };
		let dy = if y1 > y2 { !0 } else { 1 };
		for step in 0..=abs_diff(x1, x2) {
			let x = x1.wrapping_add(step.wrapping_mul(dx));
			let y = y1.wrapping_add(step.wrapping_mul(dy));
			//dbg!((x1, y1, x2, y2, dx, dy, step, x, y));
			ensure_fits(grid, Pos(x, y));
			grid[y][x] += 1;
		}
	}
}

fn count_intersections(grid: &Vec<Vec<usize>>, min: usize) -> usize {
	grid.iter()
		.flat_map(|line| line.iter().copied())
		.filter(|&c| c >= min)
		.count()
}

fn abs_diff(a: usize, b: usize) -> usize {
	if a > b {
		a - b
	} else {
		b - a
	}
}
