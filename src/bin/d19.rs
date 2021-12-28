use std::iter;
use std::thread;
use std::io::{self, BufRead, BufReader};
use std::cmp::Ordering::*;
use std::collections::HashSet;
use std::ops;
use std::sync::Arc;

use eyre::{Result, WrapErr, eyre, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct V3([i32; 3]);

impl ops::Add for V3 {
	type Output = V3;

	#[inline]
	fn add(self, V3([x2, y2, z2]): V3) -> V3 {
		let V3([x1, y1, z1]) = self;
		V3([x1 + x2, y1 + y2, z1 + z2])
	}
}

impl ops::Sub for V3 {
	type Output = V3;

	#[inline]
	fn sub(self, rhs: V3) -> V3 {
		self + -rhs
	}
}

impl ops::Neg for V3 {
	type Output = V3;

	#[inline]
	fn neg(self) -> V3 {
		let V3([x, y, z]) = self;
		V3([-x, -y, -z])
	}
}

#[derive(Debug, Clone, Copy)]
struct M33([V3; 3]);

impl From<[i32; 3]> for V3 {
	#[inline]
	fn from(v: [i32; 3]) -> V3 { V3(v) }
}

impl<T: Into<V3>> From<[T; 3]> for M33 {
	#[inline]
	fn from([x, y, z]: [T; 3]) -> M33 { M33([x.into(), y.into(), z.into()]) }
}

impl M33 {
	fn transpose(self) -> M33 {
		let M33([V3(m1), V3(m2), V3(m3)]) = self;
		[
			[m1[0], m2[0], m3[0]],
			[m1[1], m2[1], m3[1]],
			[m1[2], m2[2], m3[2]],
		].into()
	}
}

impl ops::Mul<V3> for M33 {
	type Output = V3;

	fn mul(self, rhs: V3) -> V3 {
		let M33([V3(m1), V3(m2), V3(m3)]) = self;
		let V3([x, y, z]) = rhs;
		[
			m1[0] * x + m1[1] * y + m1[2] * z,
			m2[0] * x + m2[1] * y + m2[2] * z,
			m3[0] * x + m3[1] * y + m3[2] * z,
		].into()
	}
}

impl ops::Mul for M33 {
	type Output = M33;

	fn mul(self, rhs: M33) -> M33 {
		let [v1, v2, v3] = self.0;
		let r = rhs.transpose();
		[
			r * v1,
			r * v2,
			r * v3,
		].into()
	}
}

fn rotations() -> [M33; 24] {
	let id: M33 = [
		[1, 0, 0],
		[0, 1, 0],
		[0, 0, 1],
	].into();

	let r_x: M33 = [
		[1, 0, 0],
		[0, 0, 1],
		[0, -1, 0],
	].into();

	let r_y: M33 = [
		[0, 0, -1],
		[0, 1, 0],
		[1, 0, 0],
	].into();

	let r_z = [
		[ 0, 1, 0],
		[-1, 0, 0],
		[ 0, 0, 1],
	].into();

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
			o[i*4+j] = m;
			m = r_x * m;
		}
	}

	o
}

#[inline]
fn manhattan_distance(a: V3, b: V3) -> i32 {
	let V3([x, y, z]) = a - b;
	x.abs() + y.abs() + z.abs()
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Beacon {
	coords: V3,
	neighbors: Vec<V3>,
}

#[inline]
fn neighbors_for_beacon<I: Iterator<Item=V3>>(coords: V3, all: I)
		-> Vec<V3> {
	let mut neighbors: Vec<_> = all.filter_map(|other|
		match coords == other {
			true => None,
			false => Some(other - coords),
		}).collect();
	neighbors.sort();
	neighbors
}

fn beacons_with_neighbors(beacon_coords: &[V3]) -> Vec<Beacon> {
	beacon_coords.iter().map(|&coords| {
		let neighbors =
			neighbors_for_beacon(coords, beacon_coords.iter().cloned());
		Beacon { coords, neighbors }
	}).collect()
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
			let coords = V3([
				coords.next().ok_or_else(|| eyre!("missing coord"))??,
				coords.next().ok_or_else(|| eyre!("missing coord"))??,
				coords.next().ok_or_else(|| eyre!("missing coord"))??,
			]);
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

	let mut normalized_scanners = vec![[0, 0, 0].into()];

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
								.map(|n| r * n).collect();
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

							let normalized_coords = r * candidate_beacon.coords;
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

	// for (i, &V3([x, y, z])) in normalized_beacons.iter().enumerate() {
	// 	println!("beacon {i:>3}:   {x:>5}, {y:>5}, {z:>5}");
	// }

	// for (i, &V3([x, y, z])) in normalized_scanners.iter().enumerate() {
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
