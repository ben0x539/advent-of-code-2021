use eyre::{Result, WrapErr};
use itertools::Itertools;

use std::io::{self, BufRead, BufReader};

fn main() -> Result<()> {
	let numbers: Vec<i64> = BufReader::new(io::stdin())
		.lines()
		.map(|line| Ok(line
					.wrap_err("couldn't read line")?
					.parse().wrap_err_with(|| format!("that's not a number: {:?}", line))?)
			)
		.collect::<Result<_>>()?;

	let triples = numbers.iter().tuple_windows::<(_, _, _)>();

	let counter: i64 = triples
		.clone()
		.zip(triples.clone().skip(1))
		.map(|((a1, a2, a3), (b1, b2, b3))| if a1 + a2 + a3 < b1 + b2 + b3 { 1 } else { 0 })
		.sum();

	println!("{}", counter);
	Ok(())
}
