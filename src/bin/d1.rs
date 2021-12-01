use std::io::{self, BufReader, BufRead};

fn main() {
	let mut counter = 0;

	let mut numbers = Vec::new();
	for line in BufReader::new(io::stdin()).lines() {
		let line = line.unwrap();
		let line = line.trim();
		if line == "" { continue; }
		let n: i64 = line.parse().unwrap();
		numbers.push(n);
	}

	let triples = numbers.iter().zip(numbers.iter().skip(1).zip(numbers.iter().skip(2)));

	for ((a1, (a2, a3)), (b1, (b2, b3))) in triples.clone().zip(triples.clone().skip(1)) {
		if a1+a2+a3 < b1+b2+b3 {
			counter += 1;
		}
	}

	println!("{}", counter);
}
