use itertools::Itertools;

use std::io::{self, BufReader, BufRead};

fn main() {
	let numbers: Vec<_> = BufReader::new(io::stdin()).lines()
		.filter_map(|line|
				line.ok()
					.and_then(|line| line.trim().parse::<i64>().ok()))
		.collect();

	let triples = numbers.iter().tuple_windows();

	let counter: i64 = triples.clone().zip(triples.clone().skip(1))
		.map(|((a1, a2, a3), (b1, b2, b3))|
			if a1+a2+a3 < b1+b2+b3 { 1 } else { 0 })
		.sum();

	println!("{}", counter);
}
