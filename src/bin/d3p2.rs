use std::io::{self, BufRead, BufReader};
use eyre::{Result, bail, eyre};

fn main() -> Result<()> {
    let lines: Vec<String> = BufReader::new(io::stdin())
         .lines()
         .map(|line| Ok(line?))
         .collect::<Result<_>>()?;

    if lines.is_empty() {
        bail!("input empty");
    }
    if lines.iter().any(|l| l.len() != lines[0].len()) {
        bail!("input not rectangular");
    }

    let oxygen_gen_rating = get_rating(lines.clone(), true)?
        .ok_or(eyre!("no oxygen gen rating"))?;
    let co2_scrubber_rating = get_rating(lines.clone(), false)?
        .ok_or(eyre!("no co2 scrubber rating"))?;

    dbg!(&oxygen_gen_rating, &co2_scrubber_rating);

    let oxygen_gen_rating = i32::from_str_radix(&oxygen_gen_rating, 2)?;
    let co2_scrubber_rating = i32::from_str_radix(&co2_scrubber_rating, 2)?;

    eprintln!("{} * {} = {}", oxygen_gen_rating, co2_scrubber_rating,
        oxygen_gen_rating * co2_scrubber_rating);

    Ok(())
}

fn get_rating<'a>(mut lines: Vec<String>, want_most_common: bool)
        -> Result<Option<String>> {
    for i in 0..lines[0].len() {
        let mut lines_one = Vec::new();
        let mut lines_zero = Vec::new();

        for line in lines {
            match line.as_bytes()[i] {
                b'1' => &mut lines_one,
                b'0' => &mut lines_zero,
                _ => bail!("bad digit in {:?}", line),
            }.push(line);
        }

        lines = if want_most_common == (lines_one.len() >= lines_zero.len()) {
            lines_one
        } else {
            lines_zero
        };

        if lines.len() == 1 {
            return Ok(lines.pop());
        }
    }

    return Ok(None);
}
