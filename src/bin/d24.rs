use std::io::{self, BufRead, BufReader};
use std::fs;
use std::fmt;
use std::default::Default;

use eyre::{Result, eyre, bail};

#[derive(Debug, Clone, Copy)]
struct Var {
	index: i32,
}

impl fmt::Display for Var {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", b"wxyz"[self.index as usize] as char)
	}
}

#[derive(Debug)]
enum Val {
	Var(Var),
	Lit(i64),
}

impl fmt::Display for Val {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Val::Var(v) => write!(f, "{v}"),
			Val::Lit(v) => write!(f, "{v}"),
		}
	}
}

#[derive(Debug)]
enum Ins {
	Inp(Var),
	Add(Var, Val),
	Mul(Var, Val),
	Div(Var, Val),
	Mod(Var, Val),
	Eql(Var, Val),
}

#[derive(Debug, Default)]
struct State {
	vars: [i64; 4],
}

impl State {
	fn apply<'a, I, J>(&mut self, instrs: I, input: J) -> Result<i64>
			where
				I: IntoIterator<Item = &'a Ins>,
				J: IntoIterator<Item = i64> {
		let mut input = input.into_iter();
		for ins in instrs {
			match ins {
				Ins::Inp(var) => {
					let i = input.next().ok_or_else(|| eyre!("unexpected eof"))?;
					*self.var_mut(var) = i;
				}
				Ins::Add(var, val) => *self.var_mut(var) += self.val(val),
				Ins::Mul(var, val) => *self.var_mut(var) *= self.val(val),
				Ins::Div(var, val) => *self.var_mut(var) /= self.val(val),
				Ins::Mod(var, val) => *self.var_mut(var) %= self.val(val),
				Ins::Eql(var, val) => {
					let rhs = self.val(val);
					let lhs = self.var_mut(var);
					*lhs = if *lhs == rhs { 1 } else { 0 };
				}
			}
		}

		Ok(*self.var(&"z".parse()?))
	}

	fn var_mut(&mut self, var: &Var) -> &mut i64 {
		&mut self.vars[var.index as usize]
	}

	fn var(&self, var: &Var) -> &i64 {
		&self.vars[var.index as usize]
	}

	fn val(&self, val: &Val) -> i64 {
		match val {
			Val::Lit(v) => *v,
			Val::Var(v) => *self.var(v),
		}
	}
}

impl std::str::FromStr for Var {
	type Err = eyre::Report;
	fn from_str(s: &str) -> Result<Var> {
		let index = match s {
			"w" => 0,
			"x" => 1,
			"y" => 2,
			"z" => 3,
			_ => bail!("bad var: {}", s),
		};

		Ok(Var { index })
	}
}

impl std::str::FromStr for Val {
	type Err = eyre::Report;
	fn from_str(s: &str) -> Result<Val> {
		if s.is_empty() {
			bail!("empty val");
		}

		if s.as_bytes()[0].is_ascii_alphabetic() {
			Ok(Val::Var(s.parse()?))
		} else {
			Ok(Val::Lit(s.parse()?))
		}
	}
}

impl std::str::FromStr for Ins {
	type Err = eyre::Report;
	fn from_str(s: &str) -> Result<Ins> {
		let mut iter = s.split_whitespace();
		use Ins::*;
		let ins = match (iter.next(), iter.next(), iter.next(), iter.next()) {
			(Some("inp"), Some(a), None, _) => Inp(a.parse()?),
			(Some("add"), Some(a), Some(b), None) => Add(a.parse()?, b.parse()?),
			(Some("mul"), Some(a), Some(b), None) => Mul(a.parse()?, b.parse()?),
			(Some("div"), Some(a), Some(b), None) => Div(a.parse()?, b.parse()?),
			(Some("mod"), Some(a), Some(b), None) => Mod(a.parse()?, b.parse()?),
			(Some("eql"), Some(a), Some(b), None) => Eql(a.parse()?, b.parse()?),
			_ => bail!("bad instruction: {}", s),
		};

		Ok(ins)
	}
}

fn main() -> Result<()> {
	let mut program: Vec<Ins> = Vec::new();

	for line in BufReader::new(io::stdin()).lines() {
		let line = line?;

		program.push(line.parse()?);
	}

	////let r = state.apply(&program, [10])?;
	//for number in (11111111111111..99999999999999u64).rev() {
	//	let digits = number.to_string().into_bytes();
	////loop {
	//	//let mut digits = env::args().skip(1).next().unwrap();
	//	//io::stdin().read_line(&mut digits)?;
	//	//let number: i64 = digits.parse()?;
	//	//let digits = digits.into_bytes();
	//	if number % 1000000 == 0 {
	//		dbg!(number);
	//	}
	//	if digits.iter().any(|&c| c == b'0') {
	//		continue;
	//	}

	//	let mut state = State::default();
	//	let r = state.apply(&program, digits.iter().map(|&c| (c - b'0') as _))?;
	//	if r == 0 {
	//		println!("{number}");
	//		break;
	//	}

	//	//println!("{} {:x} => {} {:x}", number, number, r, r);
	//	//for r in [number-r, number+r, number*r, (number as f64 / r as f64) as i64, r * r, (number / r) * (number / r)] {
	//	//	println!("  {}", r);
	//	//}
	//	//break;
	//}

	let mut f = fs::File::create("out.rs")?;
	compile(&mut f, &program)?;
	
	Ok(())
}

fn compile(mut w: impl io::Write, program: &[Ins]) -> Result<()> {
	writeln!(w, "{}", "fn main() {
		for number in (11111111111111..99999999999999i64).rev() {
			let digits = number.to_string().into_bytes();
			if number % 100000000 == 0 {
				dbg!(number);
			}
			if digits.iter().any(|&c| c == b'0') {
				continue;
			}
			let mut input = &digits[..];
			let mut w = 0i64;
			let mut x = 0i64;
			let mut y = 0i64;
			let mut z = 0i64;
			")?;

	for ins in program {
		match ins {
			Ins::Inp(var) => writeln!(w, "{} = input[0] as i64; input = &input[1..];", var),
			Ins::Add(var, val) => writeln!(w, "{} += {};", var, val),
			Ins::Mul(var, val) => writeln!(w, "{} *= {};", var, val),
			Ins::Div(var, val) => writeln!(w, "{} /= {};", var, val),
			Ins::Mod(var, val) => writeln!(w, "{} %= {};", var, val),
			Ins::Eql(var, val) => writeln!(w, "{} = if {} == {} {{ 1 }} else {{ 0 }};", var, var, val),
		}?;
	}
	writeln!(w, "{}", r#"if z == 0 {
				println!("{}", z);
			}
		}
	}"#)?;

	Ok(())
}

/*
fn program() {
	let mut z = 0;

	let w = inp();

	if z % 26 + 13 != w {
		z /= 1;
		z += z * 26 + w;
	} else {
		z /= 1;
	};

	let w = inp();
	if w != (z % 26) + v2 {
		z /= v1;
		z = z * 26 + w + v3;
	} else {
		z /= v1;
	}

}
*/

/*
i1+0
i2+3
i3+8
i4 = i3+8-5 = i3+3
i5+13
i6+9
i7+6
i8=i7+6-14=i7-8
i9=i6+9-8=i6+1
i10+2
i11=i10+2-0=i10+2
i12=i5+13-5=i5+8
i13=i2+3-9=i2-6
i14=i1+0-1=i1-1

i1=9
i2=9
i3=6
i4=9
i5=1
i6=8
i7=9
i8=1
i9=9
i10=7
i11=9
i12=9
i13=3
i14=8

i1=2
i2=7
i3=1
i4=4
i5=1
i6=1
i7=9
i8=1
o9=2
i10=1
i11=3
i12=9
i13=1
i14=1
*/

/*
i1
z = i1+0
i2
z = i1+0, i2+3
i3
z = i1+0, i2+3, i3+8
i4=i3+3
z = i1+0, i2+3
z = i1+0, i2+3, i5+13
z = i1+0, i2+3, i5+13, i6+9
z = i1+0, i2+3, i5+13, i6+9, i7+1
i8 = i7-7
z = i1+0, i2+3, i5+13, i6+9
z = i1+0, i2+3, i5+13, i6+9, i9+2
i10 = i9+2
z = i1+0, i2+3, i5+13, i6+9
z = i1+0, i2+3, i5+13, i6+9
i11 = i6+4
z = i1+0, i2+3, i5+13
i12 = i5+4
z = i1+0, i2+3
i13 = i2-7
*/
