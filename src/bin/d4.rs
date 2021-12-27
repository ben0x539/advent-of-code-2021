use std::io::{self, BufReader, BufRead};

use eyre::{Result, eyre, bail};

type Square = Vec<Vec<Option<i32>>>;

fn main() -> Result<()> {
	let lines: Vec<_> = BufReader::new(io::stdin()).lines()
		.collect::<Result<_, _>>()?;
	let mut lines = lines.into_iter();
	let drawings_line = lines.next()
		.ok_or_else(|| eyre!("drawn numbers missing"))?;
	let drawings: Vec<i32> = drawings_line.split(',')
		.map(|s| Ok(s.parse()?))
		.collect::<Result<Vec<i32>>>()?;

	if !lines.next().ok_or_else(|| eyre!("unexpected eof"))?.is_empty() {
		bail!("unexpected non-empty line");
	}

	let mut squares = Vec::new();
	loop {
		let square: Square = (&mut lines)
			.take_while(|line| !line.is_empty())
			.map(|line| line.split_ascii_whitespace()
				.map(|s| s.parse().map(Some))
				.collect::<Result<_, _>>())
			.collect::<Result<_, _>>()?;
		if square.is_empty() {
			break;
		}

		squares.push(square);
	}

	// dbg!(&drawings);
	// dbg!(&squares);

	let dim = squares[0].len();
	let non_square = squares.iter().find(|square|
			square.len() != dim || square.iter().any(|line| line.len() != dim));
	if let Some(non_square) = non_square {
		bail!("non-square square: {:?}", non_square);
	}

	let score = get_first_winning_score(&mut squares, drawings.iter().copied())
		.ok_or_else(|| eyre!("nobody won"))?;
	println!("{score}");

	Ok(())
}

fn get_first_winning_score<I>(squares: &mut Vec<Square>, drawings: I)
			-> Option<i32>
		where I: Iterator<Item=i32> {
	for number in drawings {
		for square in &mut *squares {
			mark_number_on_square(square, number);

			if square_won(square) {
				let sum = square_score(square);
				return Some(sum * number);
			}
		}
	}

	None
}


fn square_won(square: &Square) -> bool {
	(0..square.len()).any(|i|
		square[i].iter().all(|cell| cell.is_none())
			|| square.iter().map(|line| line[i]).all(|cell| cell.is_none()))
}

fn square_score(square: &Square) -> i32 {
	square.iter().flat_map(|line| line.into_iter())
		.filter_map(|cell| cell.as_ref())
		.sum()
}

fn mark_number_on_square(square: &mut Square, number: i32) {
	for line in square {
		for cell in line {
			if cell == &Some(number) {
				*cell = None;
			}
		}
	}
}
