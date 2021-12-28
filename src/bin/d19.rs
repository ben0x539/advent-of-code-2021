use std::iter;
use std::thread;
use std::io::{self, BufRead, BufReader};
use std::cmp::Ordering::*;
use std::collections::HashSet;
use std::ops;
use std::sync::Arc;

use eyre::{Result, WrapErr, eyre, bail};

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
struct Vec3 {
	x: i32,
	y: i32,
	z: i32,
}

impl Vec3 {
	#[inline]
	fn rotate(self, r: M33) -> Vec3 {
		let v = V3([self.x, self.y, self.z]);
		let V3([x, y, z]) = r * v;
		Vec3 { x, y, z }
	}
}

impl From<(i32, i32, i32)> for Vec3 {
	#[inline]
	fn from((x, y, z): (i32, i32, i32)) -> Vec3 { Vec3 { x, y, z } }
}

impl From<Vec3> for (i32, i32, i32) {
	#[inline]
	fn from(Vec3 { x, y, z }: Vec3) -> (i32, i32, i32) { (x, y, z) }
}

impl ops::Add for Vec3 {
	type Output = Vec3;

	#[inline]
	fn add(self, rhs: Vec3) -> Vec3 {
		Vec3 {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl ops::Sub for Vec3 {
	type Output = Vec3;

	#[inline]
	fn sub(self, rhs: Vec3) -> Vec3 {
		self + -rhs
	}
}

impl ops::Neg for Vec3 {
	type Output = Vec3;

	#[inline]
	fn neg(self) -> Vec3 {
		Vec3 {
			x: -self.x,
			y: -self.y,
			z: -self.z,
		}
	}
}

#[inline]
fn manhattan_distance(a: Vec3, b: Vec3) -> i32 {
	#[inline]
	fn d(a: i32, b: i32) -> i32 { i32::abs(a-b) }

	d(a.x, b.x) + d(a.y, b.y) + d(a.z, b.z)
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Beacon {
	coords: Vec3,
	neighbors: Vec<Vec3>,
}

#[inline]
fn neighbors_for_beacon<I: Iterator<Item=Vec3>>(coords: Vec3, all: I)
		-> Vec<Vec3> {
	let mut neighbors: Vec<_> = all.filter_map(|other|
		match coords == other {
			true => None,
			false => Some(other - coords),
		}).collect();
	neighbors.sort();
	neighbors
}

fn beacons_with_neighbors(beacon_coords: &[Vec3]) -> Vec<Beacon> {
	beacon_coords.iter().map(|&coords| {
		let neighbors =
			neighbors_for_beacon(coords, beacon_coords.iter().cloned());
		Beacon { coords, neighbors }
	}).collect()
}

#[derive(Debug, Clone, Copy)]
struct V3([i32; 3]);
#[derive(Debug, Clone, Copy)]
struct M33([V3; 3]);

impl M33 {
	fn transpose(self) -> M33 {
		M33([
			V3([self.0[0].0[0], self.0[1].0[0], self.0[2].0[0]]),
			V3([self.0[0].0[1], self.0[1].0[1], self.0[2].0[1]]),
			V3([self.0[0].0[2], self.0[1].0[2], self.0[2].0[2]]),
		])
	}
}

impl ops::Mul<V3> for M33 {
	type Output = V3;

	fn mul(self, rhs: V3) -> V3 {
		V3([
			self.0[0].0[0] * rhs.0[0] +
			self.0[0].0[1] * rhs.0[1] +
			self.0[0].0[2] * rhs.0[2],
			self.0[1].0[0] * rhs.0[0] +
			self.0[1].0[1] * rhs.0[1] +
			self.0[1].0[2] * rhs.0[2],
			self.0[2].0[0] * rhs.0[0] +
			self.0[2].0[1] * rhs.0[1] +
			self.0[2].0[2] * rhs.0[2],
		])
	}
}

impl ops::Mul for M33 {
	type Output = M33;

	fn mul(self, rhs: M33) -> M33 {
		let r = rhs.transpose();
		M33([
			r * self.0[0],
			r * self.0[1],
			r * self.0[2],
		])
	}
}

#[inline]
fn rotations() -> [M33; 24] {
	let id = M33([
		V3([1, 0, 0]),
		V3([0, 1, 0]),
		V3([0, 0, 1]),
	]);

	let r_x = M33([
		V3([1, 0, 0]),
		V3([0, 0, 1]),
		V3([0, -1, 0]),
	]);

	let r_y = M33([
		V3([0, 0, -1]),
		V3([0, 1, 0]),
		V3([1, 0, 0]),
	]);

	let r_z = M33([
		V3([0, 1, 0]),
		V3([-1, 0, 0]),
		V3([0, 0, 1]),
	]);

	let r1s: &[M33] = &[
		id,
		r_y,
		r_y * r_y,
		r_y * r_y * r_y,
		r_z,
		r_z * r_z * r_z,
	];

	let mut o = [id; 24];

	for i in 0..r1s.len() {
		let mut m = r1s[i];

		for j in 0..4 {
			//eprintln!("{:?}", m);
			o[i*4+j] = m;
			m = r_x * m;
		}
	}

	o
}

// xs, ys need to be sorted
#[inline]
fn count_matches<'a, T, I>(xs: I, ys: I) -> u32
		where T: 'a+Ord, I: IntoIterator<Item=&'a T> {
	let mut xs = xs.into_iter();
	let mut ys = ys.into_iter();
	let mut x = xs.next();
	let mut y = ys.next();
	let mut count = 0;
	loop {
		match x.and_then(|x| y.map(|y| x.cmp(&y))) {
			None => return count,
			Some(Less) => x = xs.next(),
			Some(Greater) => y = ys.next(),
			Some(Equal) => {
				x = xs.next();
				y = ys.next();
				count += 1;
			}
		}
	}
}

fn main() -> Result<()> {
	let mut scanner_beacon_coords = Vec::new();
	for line in BufReader::new(io::stdin()).lines() {
		let line = line?;
		if line.starts_with("--- ") {
			scanner_beacon_coords.push(Vec::new());
		} else if line.is_empty() {
			continue;
		} else {
			let mut coords = line.split(',')
				.map(|s| s.parse::<i32>()
					.with_context(|| eyre!("weird coord in {}", line)));
			let coords = Vec3 {
				x: coords.next().ok_or_else(|| eyre!("missing coord"))??,
				y: coords.next().ok_or_else(|| eyre!("missing coord"))??,
				z: coords.next().ok_or_else(|| eyre!("missing coord"))??,
			};
			scanner_beacon_coords
				.last_mut().ok_or_else(|| eyre!("wtf no scanner"))?
				.push(coords);
		}
	}

	let mut remaining_areas: Vec<Vec<Beacon>> =
		scanner_beacon_coords.iter().map(|beacon_coords|
			beacons_with_neighbors(beacon_coords)).collect();

	let initial_area = remaining_areas.pop()
		.ok_or_else(|| eyre!("no scanners, rip"))?;

	let mut normalized_beacons: HashSet<_> =
		initial_area.iter().map(|beacon| beacon.coords).collect();

	let mut reference_areas = Arc::new(vec![initial_area]);

	let mut normalized_scanners = vec![(0, 0, 0).into()];

	let rotations = rotations();

	while !remaining_areas.is_empty() {
		let mut new_normalized_areas = Vec::new();
		//eprintln!("unconnected scanners left: {}", remaining_areas.len());

		// move all remaining areas into individual threads, leaving
		// remaining_areas empty
		let results = remaining_areas.drain(..).map(|candidate_area| {
			let reference_areas = reference_areas.clone();
			thread::spawn(move || {
				for candidate_beacon in &candidate_area {
					for r in rotations {
						let mut candidate_neighbors: Vec<_> =
							candidate_beacon.neighbors.iter().cloned()
								.map(|n| n.rotate(r)).collect();
						candidate_neighbors.sort();

						for reference_beacon in reference_areas.iter().flatten() {
							let matches = count_matches(
								&reference_beacon.neighbors, &candidate_neighbors);

							if matches < 11 {
								continue;
							}

							// reconstruct normalized coordinates from the
							// neighbor offsets we just normalized
							let new_normalized_area: Vec<Beacon> =
								beacons_with_neighbors(
									&candidate_neighbors.iter()
										.map(|&c| c + reference_beacon.coords)
										.chain(iter::once(reference_beacon.coords))
										.collect::<Vec<_>>());

							let normalized_coords = candidate_beacon.coords.rotate(r);
							let normalized_scanner =
								reference_beacon.coords - normalized_coords;
							return (None, Some((new_normalized_area, normalized_scanner)));
						}
					}
				}
				return (Some(candidate_area), None);
			})
		}).collect::<Vec<_>>().into_iter()
			.map(|t| t.join().unwrap());

		for (maybe_remaining, maybe_result) in results {
			// if the thread normalized things, record them and use the
			// normalized beacons in the next generation of reference areas
			if let Some((new_normalized, new_scanner)) = maybe_result {
				normalized_beacons.extend(new_normalized.iter()
					.map(|beacon| beacon.coords));
				new_normalized_areas.push(new_normalized);
				normalized_scanners.push(new_scanner);
			}

			// if the thread didnn't normalize things, put the
			// area back for the next round
			if let Some(remaining) = maybe_remaining {
				remaining_areas.push(remaining);
			}
		}

		if new_normalized_areas.is_empty() {
			bail!("rip, {} unmatched scanners left", remaining_areas.len());
		}

		reference_areas = Arc::new(new_normalized_areas);
	}

	// for (i, &Vec3 { x, y, z }) in normalized_beacons.iter().enumerate() {
	// 	println!("beacon {i:>3}:   {x:>5}, {y:>5}, {z:>5}");
	// }

	// for (i, &Vec3 { x, y, z }) in normalized_scanners.iter().enumerate() {
	// 	println!("scanner {i:>3}:   {x:>5}, {y:>5}, {z:>5}");
	// }

	println!("beacons found: {}", normalized_beacons.len());
	//println!("scanners found: {}", normalized_scanners.len());

	let max_distance = normalized_scanners.iter().flat_map(|&s1|
			normalized_scanners.iter().map(move |&s2|
				manhattan_distance(s1, s2)))
		.max().unwrap();
	println!("max distance: {max_distance}");

	Ok(())
}
