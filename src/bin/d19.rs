use std::iter;
use std::io::{self, BufRead, BufReader};
use std::cmp::Ordering::*;
use std::collections::HashSet;
use std::ops;

use eyre::{Result, WrapErr, eyre, bail};

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
struct Vec3 {
	x: i32,
	y: i32,
	z: i32,
}

type Rotation = fn((i32, i32, i32)) -> (i32, i32, i32);

impl Vec3 {
	#[inline]
	fn rotate(self, r: Rotation) -> Vec3 {
		r(self.into()).into()
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

const ROTATIONS: &[Rotation] = &[
	|(x, y, z)| ( x,  y,  z),
	|(x, y, z)| ( x,  z, -y), // rotation around +x
	|(x, y, z)| ( x, -y, -z),
	|(x, y, z)| ( x, -z,  y),

	|(x, y, z)| (-x,  y, -z),
	|(x, y, z)| (-x, -y,  z),
	|(x, y, z)| (-x, -z, -y),
	|(x, y, z)| (-x,  z,  y),

	|(x, y, z)| ( y,  z,  x), 
	|(x, y, z)| ( y, -z, -x),
	|(x, y, z)| ( y,  x, -z),
	|(x, y, z)| ( y, -x,  z), // rotation around +z

	|(x, y, z)| (-y,  x,  z),
	|(x, y, z)| (-y, -x, -z),
	|(x, y, z)| (-y, -z,  x),
	|(x, y, z)| (-y,  z, -x),

	|(x, y, z)| ( z,  x,  y),
	|(x, y, z)| ( z, -x, -y),
	|(x, y, z)| ( z, -y,  x),
	|(x, y, z)| ( z,  y, -x),

	|(x, y, z)| (-z, -x,  y),
	|(x, y, z)| (-z,  x, -y),
	|(x, y, z)| (-z,  y,  x), // rotation around +y
	|(x, y, z)| (-z, -y, -x),
];

// let permutations: &[fn((i32, i32, i32)) -> (i32, i32, i32)] = &[
// 	|(x, y, z)| (x, y, z),
// 	|(x, y, z)| (y, z, x),
// 	|(x, y, z)| (z, x, y),
// 	|(x, y, z)| (x, z, y),
// 	|(x, y, z)| (z, y, x),
// 	|(x, y, z)| (y, x, z),
// ];

// let flips: &[fn((i32, i32, i32)) -> (i32, i32, i32)] = &[
// 	|(x, y, z)| (x, y, z),
// 	|(x, y, z)| (x, y, -z),
// 	|(x, y, z)| (x, -y, z),
// 	|(x, y, z)| (x, -y, -z),
// 	|(x, y, z)| (-x, y, z),
// 	|(x, y, z)| (-x, y, -z),
// 	|(x, y, z)| (-x, -y, z),
// 	|(x, y, z)| (-x, -y, -z),
// ];


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

#[inline]
fn retain_unordered<T, F: FnMut(&mut T) -> bool>(v: &mut Vec<T>, mut f: F) {
	let mut i = 0;
	while i < v.len() {
		match f(&mut v[i]) {
			true => { i += 1; }
			false => { v.swap_remove(i); }
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

	let mut reference_areas = vec![initial_area];

	let mut normalized_scanners = vec![(0, 0, 0).into()];

	let mut new_normalized_areas = Vec::new();
	let mut candidate_neighbors = Vec::new();
	while !remaining_areas.is_empty() {
		new_normalized_areas.truncate(0);
		//eprintln!("unconnected scanners left: {}", remaining_areas.len());

		retain_unordered(&mut remaining_areas, |candidate_area| {
			for candidate_beacon in &*candidate_area {
				for &r in ROTATIONS {
					candidate_neighbors.truncate(0);
					candidate_neighbors.extend(
						candidate_beacon.neighbors.iter().cloned()
							.map(|n| n.rotate(r)));
					candidate_neighbors.sort();

					for reference_beacon in reference_areas.iter().flatten() {
						let matches = count_matches(
							&reference_beacon.neighbors, &candidate_neighbors);

						if matches < 11 {
							continue;
						}

						normalized_beacons.extend(
							candidate_neighbors.iter()
								.map(|&neighbor| reference_beacon.coords + neighbor));

						let normalized_beacon_coords: Vec<_> =
							candidate_neighbors.iter()
								.map(|&c| c + reference_beacon.coords)
								.chain(iter::once(reference_beacon.coords))
								.collect();
						new_normalized_areas.push(
							beacons_with_neighbors(&normalized_beacon_coords));

						let normalized_coords = candidate_beacon.coords.rotate(r);
						normalized_scanners.push(
							reference_beacon.coords - normalized_coords);
						return false;
					}
				}
			}

			return true;
		});

		if new_normalized_areas.is_empty() {
			bail!("rip, {} unmatched scanners left", remaining_areas.len());
		}

		reference_areas.truncate(0);
		reference_areas.extend(new_normalized_areas.drain(..));
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
