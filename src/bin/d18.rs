use std::ops;
use std::fmt;
use std::str;
use std::io::{self, BufRead, BufReader};

use eyre::{Result, Report, bail};

#[derive(Debug, Clone)]
enum Node {
	Number(i32),
	Pair(Box<Node>, Box<Node>),
}

use Node::*;

impl Node {
	fn maybe_reduce(&mut self) -> bool {
		let mut did_anything = false;

		while self.maybe_explode() || self.maybe_split() {
			did_anything = true;
			//eprintln!("   reducing {}", self);
		}

		did_anything
	}

	fn maybe_explode(&mut self) -> bool {
		self.maybe_explode_inner(0).is_some()
	}

	fn maybe_explode_inner(&mut self, depth: i32)
			-> Option<(Option<i32>, Option<i32>)> {
		if depth >= 4 {
			let (l, r) = match self {
				Number(_) => return None,
				Pair(l, r) => (l, r),
			};

			if let (&Number(l), &Number(r)) = (&**l, &**r) {
				*self = Number(0);
				return Some((Some(l), Some(r)));
			}
		}

		let (l, r) = match self {
			Number(_) => return None,
			Pair(l, r) => (l, r),
		};

		if let Some((l2, r2)) =  l.maybe_explode_inner(depth + 1) {
			return Some((l2, r2.and_then(|r2| r.maybe_add_right(r2))));
		}

		if let Some((l2, r2)) = r.maybe_explode_inner(depth + 1) {
			return Some((l2.and_then(|l2| l.maybe_add_left(l2)), r2));
		}

		return None;
	}

	fn maybe_add_left(&mut self, v: i32) -> Option<i32> {
		match self {
			Number(n) => {
				*n += v;
				None
			},
			Pair(l, r) => r.maybe_add_left(v).and_then(|v| l.maybe_add_left(v)),
		}
	}

	fn maybe_add_right(&mut self, v: i32) -> Option<i32> {
		match self {
			Number(n) => {
				*n += v;
				None
			},
			Pair(l, r) => l.maybe_add_right(v).and_then(|v| r.maybe_add_right(v)),
		}
	}

	fn maybe_split(&mut self) -> bool {
		match self {
			Number(n) => {
				*n >= 10 && {
					let l = *n / 2;
					let r = *n - l;
					*self = Pair(Box::new(Number(l)), Box::new(Number(r)));
					true
				}
			}
			Pair(l, r) => l.maybe_split() || r.maybe_split()
		}
	}

	fn magnitude(&self) -> i32 {
		match self {
			&Number(n) => n,
			Pair(l, r) => l.magnitude() * 3 + r.magnitude() * 2,
		}
	}
}

struct Scanner<'a> {
	buf: &'a [u8],
}

impl Scanner<'_> {
	fn new(buf: &str) -> Scanner {
		let buf = buf.as_bytes();
		Scanner { buf }
	}

	fn expect_byte(&mut self, b: u8) -> Result<()> {
		let c = self.scan_byte()?;
		if c != b {
			bail!("unexpected character {:?}, expected {:?}", c, b);
		}
		Ok(())
	}

	fn expect_eof(&mut self) -> Result<()> {
		match self.buf.is_empty() {
			true => Ok(()),
			false => bail!("expected EOF"),
		}
	}

	fn peek_byte(&mut self) -> Result<u8> {
		match self.buf.is_empty() {
			true => bail!("unexpected EOF"),
			false => Ok(self.buf[0]),
		}
	}

	fn scan_byte(&mut self) -> Result<u8> {
		let byte = self.peek_byte()?;
		self.buf = &self.buf[1..];
		Ok(byte)
	}

	fn scan_tree(&mut self) -> Result<Node> {
		let node = self.scan_node()?;
		self.expect_eof()?;
		return Ok(node);
	}

	fn scan_node(&mut self) -> Result<Node> {
		match self.peek_byte()? {
			b'[' => self.scan_pair(),
			c if c.is_ascii_digit() => self.scan_number(),
			c => bail!("unexpected character: {:?}", c),
		}
	}

	fn scan_pair(&mut self) -> Result<Node> {
		self.expect_byte(b'[')?;
		let left = self.scan_node()?;
		self.expect_byte(b',')?;
		let right = self.scan_node()?;
		self.expect_byte(b']')?;

		Ok(Pair(Box::new(left), Box::new(right)))
	}

	fn scan_number(&mut self) -> Result<Node> {
		let mut result = 0;
		while self.peek_byte()?.is_ascii_digit() {
			result = result * 10 + (self.scan_byte().unwrap() - b'0') as i32;
		}

		Ok(Number(result))
	}
}

impl str::FromStr for Node {
	type Err = Report;

	fn from_str(s: &str) -> Result<Node> {
		Scanner::new(s).scan_tree()
	}
}

impl fmt::Display for Node {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Pair(a, b) => write!(f, "[{},{}]", a, b),
			Number(n) => write!(f, "{}", n),
		}
	}
}

impl ops::Add for Node {
	type Output = Node;

	fn add(self, rhs: Node) -> Node {
		let mut result = Pair(Box::new(self), Box::new(rhs));
		result.maybe_reduce();
		result
	}
}

impl ops::Add for &Node {
	type Output = Node;

	fn add(self, rhs: &Node) -> Node {
		self.clone() + rhs.clone()
	}
}

fn main() -> Result<()> {
	let mut numbers: Vec<Node> = Vec::new();

	for line in BufReader::new(io::stdin()).lines() {
		let line = line?;
		let mut number: Node = line.parse()?;
		eprintln!("read number: {number}");
		if number.maybe_reduce(){ 
			eprintln!("  reduce to: {number}");
		}
		numbers.push(number);
	}

	// for number in &numbers {
	// 	println!("{number}\n{number:?}\n\n");
	// }

	let mut max = None;
	
	for a in &numbers {
		for b in &numbers {
			let m = (a + b).magnitude();
			if max.is_none() || max.unwrap() < m {
				max = Some(m);
			}
		}
	}

	println!("max magnitude: {:?}", max);

	let number = numbers.into_iter().reduce(|a, b| a + b).unwrap_or(Number(0));
	println!("sum: {}\nsum magnitude: {}", number, number.magnitude());

	Ok(())
}
