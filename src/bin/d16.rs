use std::io::{self, Read};

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
	buf: &'a [u8],
	offset: u64,
	consumed: u64,
}

impl<'a> Scanner<'a> {
	fn new(buf: &'a [u8]) -> Scanner<'a> {
		let offset = 0;
		let consumed = 0;

		Scanner { buf, offset, consumed }
	}

	fn read_hex(&self) -> Result<u64> {
		let c = self.buf.first()
				.ok_or(eyre!("unexpected eof"))?
				.to_ascii_uppercase();

		let r = match c {
			x @ b'0'..=b'9' => x - b'0',
			x @ b'A'..=b'F' => x - b'A' + 10,
			x => bail!("unexpected non-hex-digit: {}", x),
		};

		Ok(r as u64)
	}

	fn scan_bits(&mut self, n: u64) -> Result<u64> {
		assert!(n <= 64);

		let mut out = 0u64;
		let mut remaining = n;

		while remaining > 0 {
			//dbg!(remaining);
			let bits_available = 4 - self.offset;
			//dbg!(bits_available);
			let bits_count = u64::min(bits_available, remaining);
			//dbg!(bits_count);
			remaining -= bits_count;
			let x = self.read_hex()?;
			//eprintln!("x={:0>4b}", x);
			let mask = !(!0u64 << bits_available);
			//eprintln!("mask={:0>64}", x);
			let x = (x & mask) >> (bits_available - bits_count);
			//eprintln!("x={:0>4b}", x);
			self.offset += bits_count;
			self.consumed += bits_count;
			assert!(self.offset <= 4);
			if self.offset == 4 {
				self.buf = &self.buf[1..];
				self.offset = 0;
			}
			out <<= bits_count;
			out |= x;
			//eprintln!("out={:0>64b}", x);
		}

		//eprintln!("scan_bits({}) -> {:0>len$b}", n, out, len = n as usize);

		Ok(out)
	}

	fn scan_literal(&mut self) -> Result<u64> {
		let mut out = 0u64;

		loop {
			let last = self.scan_bits(1)? == 0;
			out <<= 4;
			out |= self.scan_bits(4)?;
			if last { break; }
		}

		Ok(out)
	}

	fn scan_operator(&mut self) -> Result<Vec<Packet>> {
		let length_type_id = self.scan_bits(1)?;
		//dbg!(length_type_id);
		match length_type_id {
			0 => self.scan_operator_bit_length(),
			1 => self.scan_operator_packet_count(),
			n => bail!("unexpected length type id: {}", n),
		}
	}

	fn scan_operator_bit_length(&mut self) -> Result<Vec<Packet>> {
		let bit_length = self.scan_bits(15)?;
		let end = self.consumed + bit_length;
		let mut packets = Vec::new();

		while self.consumed < end {
			packets.push(self.scan_packet()?);
		}
		assert_eq!(self.consumed, end);

		Ok(packets)
	}

	fn scan_operator_packet_count(&mut self) -> Result<Vec<Packet>> {
		let packet_count = self.scan_bits(11)?;
		let mut packets = Vec::new();

		for _ in 0..packet_count {
			packets.push(self.scan_packet()?);
		}

		Ok(packets)
	}

	fn scan_packet(&mut self) -> Result<Packet> {
		let version = self.scan_bits(3)?;
		let packet_type = self.scan_bits(3)?;
		//dbg!(self.consumed, version, packet_type);

		let payload = match packet_type {
			4 => Payload::Literal(self.scan_literal()?),
			n => Payload::Operator(n, self.scan_operator()?),
		};

		//dbg!(version, &payload);

		Ok(Packet { version, payload })
	}
}

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;
	let mut scanner = Scanner::new(input.trim().as_bytes());
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
