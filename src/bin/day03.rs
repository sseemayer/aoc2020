use std::fs::File;

use snafu::{ResultExt, Snafu};

use aoc2020::map::{Coords, MapError, Tile, TileMap};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("I/O error on '{}': {}", filename, source))]
    Io {
        filename: String,
        source: std::io::Error,
    },

    #[snafu(display("Map error: {}", source))]
    Map { source: MapError },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum MapTile {
    Tree,
    PathEmpty,
    PathTree,
}

impl Tile for MapTile {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '#' => Some(MapTile::Tree),
            _ => None,
        }
    }
}

impl std::fmt::Display for MapTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MapTile::Tree => "#",
                MapTile::PathEmpty => "O",
                MapTile::PathTree => "X",
            }
        )
    }
}

fn count_trees(map: &mut TileMap<MapTile>, di: usize, dj: usize) -> usize {
    let (imin, imax, jmin, jmax) = map.get_extent();
    let mut i = 0;
    let mut j = 0;
    let mut hit_trees = 0;

    while i <= imax {
        let current = { map.get(&(i, j)).cloned() };

        map.insert(
            (i, j),
            match current {
                None => MapTile::PathEmpty,
                Some(MapTile::Tree) => {
                    hit_trees += 1;
                    MapTile::PathTree
                }
                _ => unreachable!(),
            },
        );

        i += di;
        j = (j + dj) % (jmax + 1);
    }

    hit_trees
}

fn main() -> Result<()> {
    let filename = "data/day03/input";
    let mut f = File::open(filename).context(Io {
        filename: filename.to_string(),
    })?;

    let map = TileMap::<MapTile>::read(&mut f).context(Map)?;
    let recipes = vec![(1, 1), (1, 3), (1, 5), (1, 7), (2, 1)];
    let mut product = 1;
    for (di, dj) in recipes {
        let mut instance = map.clone();
        let hit_trees = count_trees(&mut instance, di, dj);

        println!("==== RECIPE {} right, {} down ====", dj, di);
        println!("{}", instance);
        println!("Hit trees: {}\n", hit_trees);

        product *= hit_trees;
    }
    println!("Final answer is {}", product);

    Ok(())
}
