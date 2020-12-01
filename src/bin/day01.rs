use std::fs::File;
use std::io::{BufRead, BufReader};

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("I/O error on '{}': {}", filename, source))]
    Io {
        filename: String,
        source: std::io::Error,
    },

    #[snafu(display("Number parsing error: {}", source))]
    ParseNumber { source: std::num::ParseIntError },
}

type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let f = File::open("data/day01/input").context(Io {
        filename: "data/day01/input".to_string(),
    })?;

    let br = BufReader::new(f);

    let lines: Vec<String> = br
        .lines()
        .collect::<std::result::Result<Vec<String>, std::io::Error>>()
        .context(Io {
            filename: "data/day01/input".to_owned(),
        })?;

    let numbers: Vec<u64> = lines
        .iter()
        .map(|v| v.parse())
        .collect::<std::result::Result<Vec<_>, std::num::ParseIntError>>()
        .context(ParseNumber)?;

    for a in &numbers {
        for b in &numbers {
            if (a + b) == 2020 {
                println!("{} * {} = {}", a, b, a * b);
            }
        }
    }

    for a in &numbers {
        for b in &numbers {
            for c in &numbers {
                if (a + b + c) == 2020 {
                    println!("{} * {} * {} = {}", a, b, c, a * b * c);
                }
            }
        }
    }

    Ok(())
}
