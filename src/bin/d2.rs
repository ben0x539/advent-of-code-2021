use std::io::{self, BufReader, BufRead};
use std::iter::Sum;
use eyre::{Result, eyre, bail};
use derive_more::Add;

#[derive(Clone, Copy, Add)]
struct Vector(i32, i32);

impl Sum<Vector> for Vector {
	fn sum<I: Iterator<Item=Vector>>(iter: I) -> Vector {
		iter.fold(Vector(0, 0), |a, b| a + b)
	}
}

fn from_movement(s: &str) -> Result<Vector> {
	let mut words = s.split_ascii_whitespace();
	let (x, y) = match words.next() {
		Some("forward") => (1, 0),
		Some("up") => (0, -1),
		Some("down") => (0, 1),
		Some(s) => bail!("bad direction: {}", s),
		None => bail!("missing direction"),
	};

	let m: i32 = words.next()
		.ok_or(eyre!("missing magnitude"))?
		.parse()?;

	Ok(Vector(x * m, y * m))
}

fn main() -> Result<()> {
	let Vector(x, y) = BufReader::new(io::stdin())
		.lines()
		.map(|line| from_movement(&line?))
		.sum::<Result<Vector>>()?;
	println!("{}", x*y);

	Ok(())
}
