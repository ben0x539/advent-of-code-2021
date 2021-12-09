use eyre::Result;
use std::io::{self, BufRead, BufReader};

fn main() -> Result<()> {
    let mut ones = Vec::new();
    let lines: Vec<String> = BufReader::new(io::stdin())
         .lines()
         .map(|line| Ok(line?))
         .collect::<Result<_>>()?;

    for line in &lines {
        ones.resize(usize::max(line.len(), ones.len()), 0);
        for (digit, counter) in line.bytes().zip(&mut ones) {
            if digit == b'1' {
                *counter += 1;
            }
        }
    }
    dbg!(&ones);
    let most_commons: Vec<_> = ones.iter()
        .map(|&counter| if counter >= lines.len() / 2 { 1u8 } else { 0 })
        .collect();
    dbg!(&most_commons);
    let oxygen_gen_rating = get_rating(lines.clone(), &most_commons);

    dbg!(&most_commons, oxygen_gen_rating);

    let least_commons: Vec<_> = most_commons.iter()
        .map(|&digit| if digit == 1 { 0 } else { 1 })
        .collect();
    let co2_scrubber_rating = get_rating(lines.clone(), &least_commons);

    dbg!(&least_commons, co2_scrubber_rating);

    Ok(())
}

fn get_rating<'a>(mut lines: Vec<String>, desireds: &[u8]) -> Option<String> {
    for (i, &desired) in desireds.iter().enumerate() {
        let mut j = 0;
        while j < lines.len() {
            let line = &lines[j];
            let b = line.as_bytes()[i] - b'0';
            if b != desired {
                lines.remove(j);
            } else {
                j += 1;
            }
            if lines.len() == 1 {
                return lines.pop();
            }
        }
    }

    return None;
}