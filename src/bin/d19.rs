use std::io::{self, BufRead, BufReader};
use std::cmp::Ordering::*;
use std::collections::BTreeSet;

use eyre::{Result, WrapErr, eyre, bail};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Beacon {
	coords: (i32, i32, i32),
	neighbors: Vec<(i32, i32, i32)>,
}

fn beacons_from_coords(coordses: &[(i32, i32, i32)]) -> Vec<Beacon> {
	coordses.iter().map(|&coords@(x, y, z)| {
		let mut neighbors: Vec<_> = coordses.iter().filter_map(|(x2,y2,z2)|
			match (x2-x, y2-y, z2-z) {
				(0, 0, 0) => None,
				d => Some(d),
			}).collect();
		neighbors.sort();
		Beacon { coords, neighbors }
	}).collect()
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
			let coords = (
				coords.next().ok_or_else(|| eyre!("missing coord"))??,
				coords.next().ok_or_else(|| eyre!("missing coord"))??,
				coords.next().ok_or_else(|| eyre!("missing coord"))??,
			);
			scanner_beacon_coords
				.last_mut().ok_or_else(|| eyre!("wtf no scanner"))?
				.push(coords);
		}
	}

	if scanner_beacon_coords.is_empty() {
		bail!("no scanners, rip");
	}

	let mut scanner_beacons: Vec<Vec<Beacon>> =
		scanner_beacon_coords.iter().map(|coordses|
			beacons_from_coords(coordses)).collect();
		//	coordses.iter().map(|&coords@(x, y, z)| {
		//		let mut neighbors: Vec<_> = coordses.iter().map(|(x2,y2,z2)|
		//			(x2-x, y2-y, z2-z)).collect();
		//		neighbors.sort();
		//		Beacon { coords, neighbors }
		//	}).collect()
		//}).collect();

	let rx = |(x, y, z)| ( x,  z, -y), // rotation around +x
	let ry = |(x, y, z)| (-z,  y,  x), // rotation around +y
	let rz = |(x, y, z)| ( y, -x,  z), // rotation around +z

	let rotations: &[fn((i32, i32, i32)) -> (i32, i32, i32)] = &[
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

	//for b in &scanner_beacons {
	//	dbg!(b.len());
	//}

	let mut beacons_to_compare = scanner_beacons.pop().unwrap();
	beacons_to_compare.sort();
	let mut beacons_found: BTreeSet<_> =
		beacons_to_compare.iter().map(|b| b.coords).collect();
	let mut b2 =
		beacons_from_coords(&beacons_found.iter().cloned().collect::<Vec<_>>());
	b2.sort();
	assert_eq!(beacons_to_compare, b2);
	let mut scanners = vec![(0, 0, 0)];

	while !scanner_beacons.is_empty() {
		eprintln!("unconnected scanners left: {}", scanner_beacons.len());
		let mut k = 0;
		let mut success = false;
		'beacons: while k < scanner_beacons.len() {
			//dbg!(k, scanner_beacons.len(), beacons_to_compare.len());
			//eprintln!("inner len {}", scanner_beacons.len());
			let beacons2 = &scanner_beacons[k];

			for beacon1 in &beacons_to_compare {
				for beacon2 in beacons2 {
					let (x, y, z) = beacon1.coords;
					for r in rotations {
						let mut neighbors2 = beacon2.neighbors.clone();
						for c in &mut neighbors2 {
							*c = r(*c)
						}

						neighbors2.sort();
						let mut overlap = Vec::new();

						let (mut i, mut j) = (0, 0);
						while i < beacon1.neighbors.len() && j < neighbors2.len()  {
							match beacon1.neighbors[i].cmp(&neighbors2[j]) {
								Equal => {
									overlap.push((i, j));
									i += 1;
									j += 1;
								}
								Less => i += 1,
								Greater => j += 1,
							}
						}

						if overlap.len() >= 11 {
							let mut new_beacon_coords = neighbors2;

							for &(dx, dy, dz) in &new_beacon_coords {
								beacons_found.insert((x+dx, y+dy, z+dz));
							}

							let (x2, y2, z2) = beacon2.coords;
							let (x2, y2, z2) = r((-x2, -y2, -z2));
							scanners.push((x2+x, y2+y, z2+z));
							scanner_beacons.swap_remove(k);
							success = true;
							continue 'beacons;
						}
					}
				}
			}
			k += 1;
		}

		if !success {
			bail!("rip, {} unmatched scanners left", scanner_beacons.len());
		}

		beacons_to_compare =
			beacons_from_coords(&beacons_found.iter().cloned().collect::<Vec<_>>());
		//dbg!(beacons_found.len());
	}

	for (i, (x, y, z)) in beacons_found.iter().enumerate() {
		println!("beacon {i:>3}:   {x:>5}, {y:>5}, {z:>5}");
	}

	for (i, (x, y, z)) in scanners.iter().enumerate() {
		println!("scanner {i:>3}:   {x:>5}, {y:>5}, {z:>5}");
	}

	println!("beacons found: {}", beacons_found.len());
	println!("scanners found: {}", scanners.len());

	fn d(a: i32, b: i32) -> i32 { i32::abs(a-b) }

	let max_distance = scanners.iter().flat_map(|&(x1, y1, z1)|
		scanners.iter().map(move |&(x2, y2, z2)|
			d(x1, x2) + d(y1, y2) + d(z1, z2))).max().unwrap();
	println!("max distance: {max_distance}");

	Ok(())
}
