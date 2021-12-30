use std::io::{self, Read};

use bitvec::prelude::*;
use eyre::{Result, eyre, bail};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
enum ControlFlow {
	Break,
	Continue,
}

use ControlFlow::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct Packet {
	version: u64,
	payload: Payload,
}

impl Packet {
	fn visit_packets<F>(&self, mut f: F) -> Result<ControlFlow>
			where F: for<'a> FnMut(&'a Packet) -> Result<ControlFlow> {
		fn inner<G>(self_: &Packet, f: &mut G) -> Result<ControlFlow>
				where G: for<'a> FnMut(&'a Packet) -> Result<ControlFlow> {
			if f(self_)? == Break {
				return Ok(Break);
			}
			
			match &self_.payload {
				Payload::Operator(_, packets) => {
					for packet in packets {
						if inner(packet, f)? == Break {
							return Ok(Break);
						}
					}
				},
				_ => {},
			}

			Ok(Continue)
		}

		inner(self, &mut f)
	}

	fn eval(&self) -> Result<u64> {
		fn bin_pred(packets: &[u64], f: impl Fn(u64, u64) -> bool)
				-> Result<u64> {
			if packets.len() != 2 {
					bail!("binary predicate has {} packets, not 2", packets.len());
			}

			let result = f(packets[0], packets[1]);
			let result = if result { 1 } else { 0 };
			Ok(result)
		}

		let result: u64 = match self.payload {
			Payload::Literal(v) => v,
			Payload::Operator(op, ref packets) => {
				let values = packets.iter()
					.map(|p| p.eval())
					.collect::<Result<Vec<_>>>()?;
				match op {
					0 => values.iter().sum(),
					1 => values.iter().product(),
					2 => *values.iter().min().ok_or(eyre!("no packets for min"))?,
					3 => *values.iter().max().ok_or(eyre!("no packets for max"))?,
					5 => bin_pred(&values, |a, b| a > b)?,
					6 => bin_pred(&values, |a, b| a < b)?,
					7 => bin_pred(&values, |a, b| a == b)?,
					_ => bail!("unknown operation type {}", op),
				}
			}
		};

		Ok(result)
	}
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
enum Payload {
	Literal(u64),
	Operator(u64, Vec<Packet>),
}

#[derive(Debug, Clone)]
struct Scanner<'a> {
	buf: &'a BitSlice<Msb0>,
}

impl<'a> Scanner<'a> {
	fn new(buf: &'a BitSlice<Msb0>) -> Scanner<'a> {
		Scanner { buf }
	}

	fn scan_bits(&mut self, n: usize) -> u64 {
		assert!(n <= 64);
		let r = self.buf[..n].load_be();
		self.buf = &self.buf[n..];
		r
	}

	fn scan_literal(&mut self) -> Result<u64> {
		let mut out = 0u64;

		loop {
			let last = self.scan_bits(1) == 0;
			out <<= 4;
			out |= self.scan_bits(4);
			if last { break; }
		}

		Ok(out)
	}

	fn scan_operator(&mut self) -> Result<Vec<Packet>> {
		let length_type_id = self.scan_bits(1);
		//dbg!(length_type_id);
		match length_type_id {
			0 => self.scan_operator_bit_length(),
			1 => self.scan_operator_packet_count(),
			n => bail!("unexpected length type id: {}", n),
		}
	}

	fn scan_operator_bit_length(&mut self) -> Result<Vec<Packet>> {
		let bit_length = self.scan_bits(15) as usize;
		let mut inner = Scanner::new(&self.buf[..bit_length]);
		println!("inner: {:016}", &inner.buf[..16]);
		self.buf = &self.buf[bit_length..];

		let mut packets = Vec::new();
		while !inner.buf.is_empty() {
			packets.push(inner.scan_packet()?);
		}
		println!("done with inner");

		Ok(packets)
	}

	fn scan_operator_packet_count(&mut self) -> Result<Vec<Packet>> {
		let packet_count = self.scan_bits(11);
		let mut packets = Vec::new();

		for _ in 0..packet_count {
			packets.push(self.scan_packet()?);
		}

		Ok(packets)
	}

	fn scan_packet(&mut self) -> Result<Packet> {
		let version = self.scan_bits(3);
		let packet_type = self.scan_bits(3);

		let payload = match packet_type {
			4 => Payload::Literal(self.scan_literal()?),
			n => Payload::Operator(n, self.scan_operator()?),
		};

		//dbg!(version, &payload);

		Ok(Packet { version, payload })
	}
}

fn load_input(mut r: impl Read) -> Result<BitVec<Msb0>> {
	let mut bits = BitVec::<Msb0>::new();
	let mut buf = [0u8; 1024];

	loop {
		let n = match r.read(&mut buf) {
			Ok(n) if n == 0 => break,
			Ok(n) => n,
			Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
			Err(e) => bail!(e),
		};
		let buf = &buf[..n];

		for b in buf {
			let b = b.to_ascii_uppercase();

			let r = match b {
				b'0'..=b'9' => b - b'0',
				b'A'..=b'F' => b - b'A' + 10,
				_ => bail!("unexpected non-hex-digit: {}", b),
			};
			let slice = &BitSlice::<Msb0, u8>::from_element(&r)[4..8];
			bits.extend_from_bitslice(slice);
		}
	}

	Ok(bits)
}

fn main() -> Result<()> {
	let input = load_input(io::stdin())?;
	println!("{:x}", input);
	let mut scanner = Scanner::new(&input);
	let packet = scanner.scan_packet()?;
	let mut versions_sum = 0;
	dbg!(&packet);
	packet.visit_packets(|p| {
		//eprintln!("version = {}", p.version);
		versions_sum += p.version;
		Ok(Continue)
	})?;
	eprintln!("version sum = {}", versions_sum);
	eprintln!("eval => {}", packet.eval()?);
	Ok(())
}
