use std::io::{self, BufRead, BufReader};

fn digit(c: char) -> bool { c.is_ascii_digit() }
fn not_digit(c: char) -> bool { !c.is_ascii_digit() }
#[track_caller]
fn atoi(s: &str) -> i32 { s.parse().unwrap() }
fn abr(s: &str) -> Option<(i32, i32, &str)> {
	let k = s.find(']').unwrap_or(s.len());
	if s[..k].find('[').is_some() { return None; }
	let c = s.find(',').unwrap();
	let a: i32 = atoi(&s[..c]);
	let b: i32 = atoi(&s[c+1..k]);
	Some((a, b, &s[k+1..]))
}

fn add(a: &str, b: &str) -> String {
	let mut s = format!("[{},{}]", a, b);
	//println!("{s}");
	'outer:
	loop {
		let mut depth = 0;
		for i in 0..s.len() {
			match s.as_bytes()[i] {
				b']' => depth -= 1,
				b'[' => {
					depth += 1;
					if depth <= 4 { continue; }
					if let Some((a, b, r)) = abr(&s[i+1..]) {
						let mut r = r.to_string();
						s.truncate(i);
						if let Some(j) = s.rfind(digit) {
							let k = s[..j].rfind(not_digit).unwrap_or(0);
							s = format!("{}{}{}", &s[..k+1], atoi(&s[k+1..j+1]) + a, &s[j+1..]);
						}
						if let Some(j) = r.find(digit) {
							let k = r[j..].find(not_digit).unwrap_or(r.len());
							r = format!("{}{}{}", &r[..j], atoi(&r[j..][..k]) + b, &r[j+k..]);
						}
						s += "0";
						s += &r;
						//eprintln!("explode => {s}");
						continue 'outer;
					}
				}
				_ => (),
			}
		}
		let mut i = 0;
		while let Some(j) = s[i..].find(digit) {
			i += j;
			let k = s[i..].find(not_digit).unwrap_or(s[i..].len());
			if k > 1 {
				let a: i32 = atoi(&s[i..][..k]);
				s = format!("{}[{},{}]{}", &s[..i], a/2, a-a/2, &s[i..][k..]);
				//eprintln!("split => {s}");
				continue 'outer;
			}
			i += k;
		}
		return s;
	}
}

fn magnitude(s: &str) -> i32 {
	let mut i = 0;
	while let Some(j) = s[i..].find('[') {
		i += j;
		if let Some((a, b, r)) = abr(&s[i+1..]) {
			let s = format!("{}{}{}", &s[..i], a*3+b*2, r);
			return magnitude(&s);
		}
		i += 1;
	}

	return s.parse().unwrap();
}

fn main() {
	let numbers: Vec<String> =
		BufReader::new(io::stdin()).lines()
			.collect::<io::Result<_>>().unwrap();

	let mm = numbers.iter().flat_map(|a|
			numbers.iter().filter_map(move |b| match a == b {
				true => None,
				false => Some(magnitude(&add(&a, &b))),
			}))
		.max().unwrap_or(0);

	let n = numbers.into_iter().reduce(|a, b| add(&a, &b))
		.unwrap_or_else(|| "0".to_string());
	println!("{}", magnitude(&n));
	println!("{}", mm);
}
