use std::io::{self, Write};
use std::collections::BTreeSet;

fn throw(mut dx: i32, mut dy: i32, show_grid: bool) -> Option<i32> {
	// let target_x = (20, 30);
	// let target_y = (-10, -5);
	let target_x = (169, 206);
	let target_y = (-108, -68);

	let mut x = 0;
	let mut y = 0;
	let mut max_x = 0;
	let mut max_y = 0;
	let mut min_x = 0;
	let mut min_y = 0;
	let mut success = false;

	let mut trajectory = BTreeSet::new();
	trajectory.insert((0, 0));

	while (dx > 0 || x >= target_x.0) && x <= target_x.1
			&& (dy > 0 || y >= target_y.0) {
		x += dx;
		y += dy;
		dx = i32::max(dx-1, 0);
		dy -= 1;

		max_y = i32::max(max_y, y);
		max_x = i32::max(max_x, x);
		min_y = i32::min(min_y, y);
		min_x = i32::min(min_x, x);

		trajectory.insert((x, y));

		//eprintln!("{},{} {},{}", x, y, dx, dy);
		//dbg!(x >= target_x.0, x <= target_x.1, y >= target_y.0, y <= target_y.1);

		if x >= target_x.0 && x <= target_x.1 && y >= target_y.0 && y <= target_y.1 {
			success = true;
		}
	}

	//dbg!(min_y, max_y, min_x, max_x);

	if show_grid {
		let y_range = i32::min(min_y, target_y.0)..i32::max(max_y, target_y.1);
		let x_range = i32::min(min_x, target_x.0)..i32::max(max_x, target_x.1);
		println!("{}", max_y);
		for y in y_range.rev() {
			for x in x_range.clone() {
				print!("{}",
					if trajectory.contains(&(x, y)) {
						'x'
					} else if x >= target_x.0 && x <= target_x.1 && y >= target_y.0 && y <= target_y.1 {
						'#'
					} else if y == 0 {
						'-'
					} else if x % 10 == 0 || y % 10 == 0 {
						'.'
					} else {
						' '
					}
				);
			}
			if y == 0 {
				print!(" 0");
			} else if y == min_y {
				print!(" {}", max_x);
			}
			println!("");
		}
		println!("{}", min_y);
	}

	match success {
		true => Some(max_y),
		false => None,
	}
}

fn main() {
	let mut max: Option<(i32, i32, i32)> = None;
	let mut successes = 0;
	//for (dx, dy) in [(6,0), (7, -1)] {
	//	throw(dx, dy, true);
	//}
	for dx in 0..1000 {
		for dy in -1000..1000 {
			if let Some(y) = throw(dx, dy, false) {
				successes += 1;
				println!("{},{}", dx, dy);
				if max.is_none() || max.unwrap().0 < y {
					max = Some((y, dx, dy));
				}
			}
		}
	}
	if let Some((_, dx, dy)) = max {
		throw(dx, dy, true);
	}
	let _ = io::stdout().flush();
	dbg!(max);
	dbg!("{}", successes);
}
