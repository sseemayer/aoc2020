use std::collections::HashMap;

use snafu::{ResultExt, Snafu};
use std::io::{BufRead, BufReader};

pub trait IntCoord:
    num::PrimInt + num::FromPrimitive + std::fmt::Debug + std::default::Default + std::hash::Hash
{
}

impl<T> IntCoord for T where
    T: num::PrimInt
        + num::FromPrimitive
        + num::ToPrimitive
        + std::marker::Copy
        + std::fmt::Debug
        + std::default::Default
        + std::hash::Hash
{
}

pub trait MapTile: std::fmt::Display + Sized + Clone {
    fn from_char(c: char) -> Option<Self>;
}

pub trait MapCoordinate: Default + Eq + std::hash::Hash + std::fmt::Debug + Clone + Copy {
    fn elementwise_min(a: Self, b: Self) -> Self;
    fn elementwise_max(a: Self, b: Self) -> Self;

    fn get_extent<'a>(mut keys: impl Iterator<Item = Self>) -> (Self, Self) {
        let mut min = keys.next().unwrap_or(Default::default());
        let mut max = min.clone();

        for k in keys {
            min = MapCoordinate::elementwise_min(min, k);
            max = MapCoordinate::elementwise_max(max, k);
        }

        (min, max)
    }
}

impl<I> MapCoordinate for (I, I)
where
    I: IntCoord,
{
    fn elementwise_min(a: Self, b: Self) -> Self {
        (std::cmp::min(a.0, b.0), std::cmp::min(a.1, b.1))
    }

    fn elementwise_max(a: Self, b: Self) -> Self {
        (std::cmp::max(a.0, b.0), std::cmp::max(a.1, b.1))
    }
}

impl<I> MapCoordinate for (I, I, I)
where
    I: IntCoord,
{
    fn elementwise_min(a: Self, b: Self) -> Self {
        (
            std::cmp::min(a.0, b.0),
            std::cmp::min(a.1, b.1),
            std::cmp::min(a.2, b.2),
        )
    }

    fn elementwise_max(a: Self, b: Self) -> Self {
        (
            std::cmp::max(a.0, b.0),
            std::cmp::max(a.1, b.1),
            std::cmp::max(a.2, b.2),
        )
    }
}

#[derive(Debug, Snafu)]
pub enum MapError {
    #[snafu(display("I/O error: {}", source))]
    Io { source: std::io::Error },
}

type MapResult<T> = std::result::Result<T, MapError>;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Map<C: MapCoordinate, T> {
    pub data: HashMap<C, T>,
    pub fixed_extent: Option<(C, C)>,
}

impl<C: MapCoordinate, T> Map<C, T> {
    pub fn new() -> Self {
        Map {
            data: HashMap::new(),
            fixed_extent: None,
        }
    }

    /// Get the tile at a coordinate
    pub fn get(&self, coord: &C) -> Option<&T> {
        self.data.get(coord)
    }

    /// Set the tile at a coordinate
    pub fn set(&mut self, coord: C, value: T) {
        self.data.insert(coord, value);
    }

    /// Clear a coordinate from tiles
    pub fn remove(&mut self, coord: &C) {
        self.data.remove(coord);
    }

    /// Get the maximum dimension for all defined tiles
    pub fn get_extent(&self) -> (C, C) {
        if let Some(e) = &self.fixed_extent {
            e.clone()
        } else {
            C::get_extent(self.data.keys().cloned())
        }
    }

    /// Find all coordinates that match a predicate
    pub fn find_all_where<P: Fn(&C, &T) -> bool>(&self, predicate: P) -> Vec<C> {
        let mut out: Vec<C> = Vec::new();
        for (coord, tile) in self.data.iter() {
            if predicate(coord, tile) {
                out.push(coord.clone());
            }
        }
        out
    }

    /// Find a coordinate that matches a predicate
    pub fn find_one_where<P: Fn(&C, &T) -> bool>(&self, predicate: P) -> Option<C> {
        for (coord, tile) in self.data.iter() {
            if predicate(coord, tile) {
                return Some(coord.clone());
            }
        }
        None
    }
}

impl<C: MapCoordinate, T: Eq> Map<C, T> {
    /// Find all coordinates that contain a tile
    pub fn find_all(&self, pattern: &T) -> Vec<C> {
        self.find_all_where(|_, t| t == pattern)
    }

    /// Find one coordinate that contains a tile
    pub fn find_one(&self, pattern: &T) -> Option<C> {
        self.find_one_where(|_, t| t == pattern)
    }
}

////// Code for 2D maps
impl<T, I> Map<(I, I), T>
where
    T: MapTile,
    I: IntCoord,
{
    pub fn read<R: std::io::Read>(reader: &mut R) -> MapResult<Self> {
        let mut data: HashMap<(I, I), T> = HashMap::new();

        let buf_reader = BufReader::new(reader);
        for (i, line) in buf_reader.lines().enumerate() {
            for (j, c) in line.context(Io)?.chars().enumerate() {
                if let Some(t) = T::from_char(c) {
                    if let (Some(i), Some(j)) = (I::from_usize(i), I::from_usize(j)) {
                        data.insert((i, j), t);
                    }
                }
            }
        }

        Ok(Map {
            data,
            fixed_extent: None,
        })
    }

    pub fn to_vecs(&self) -> Vec<Vec<Option<T>>> {
        let (min, max) = self.get_extent();

        num::iter::range_inclusive(min.0, max.0)
            .map(|i| {
                num::iter::range_inclusive(min.1, max.1)
                    .map(|j| self.data.get(&(i, j)).cloned())
                    .collect()
            })
            .collect()
    }
}

impl<T, I> std::fmt::Display for Map<(I, I), T>
where
    T: MapTile,
    I: IntCoord,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        if self.data.is_empty() {
            return Ok(());
        }

        let (min, max) = self.get_extent();

        for i in num::iter::range_inclusive(min.0, max.0) {
            for j in num::iter::range_inclusive(min.1, max.1) {
                match self.data.get(&(i, j)) {
                    Some(t) => t.fmt(f),
                    None => write!(f, " "),
                }?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl<T, I> std::str::FromStr for Map<(I, I), T>
where
    T: MapTile,
    I: IntCoord,
{
    type Err = MapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Map::read(&mut s.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestTile(char);
    impl MapTile for TestTile {
        fn from_char(c: char) -> Option<Self> {
            if c == ' ' {
                return None;
            };
            Some(TestTile(c))
        }
    }

    impl std::fmt::Display for TestTile {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
            write!(f, "{}", self.0)
        }
    }

    #[test]
    fn test_2d_parsing() {
        let map_string = "ab \nd e";
        let map = Map::<(usize, usize), TestTile>::read(&mut map_string.as_bytes()).unwrap();

        assert_eq!(map.get_extent(), ((0, 0), (1, 2)));

        assert_eq!(
            map.to_vecs(),
            vec![
                vec![Some(TestTile('a')), Some(TestTile('b')), None],
                vec![Some(TestTile('d')), None, Some(TestTile('e'))],
            ]
        )
    }

    #[test]
    fn test_2d_editing() {
        let mut map: Map<(usize, usize), TestTile> = Map::new();

        assert_eq!(map.get(&(1, 2)), None);
        map.set((1, 2), TestTile('a'));
        assert_eq!(map.get(&(1, 2)), Some(&TestTile('a')));

        map.set((4, 1), TestTile('c'));
        assert_eq!(map.get_extent(), ((1, 1), (4, 2)));

        map.set((8, 8), TestTile('d'));
        assert_eq!(map.get_extent(), ((1, 1), (8, 8)));

        map.remove(&(8, 8));
        assert_eq!(map.get_extent(), ((1, 1), (4, 2)));

        assert_eq!(
            map.to_vecs(),
            vec![
                vec![None, Some(TestTile('a'))],
                vec![None, None],
                vec![None, None],
                vec![Some(TestTile('c')), None],
            ]
        )
    }

    #[test]
    fn test_2d_display() {
        let map_string = "ab \nd e";
        let map = Map::<(usize, usize), TestTile>::read(&mut map_string.as_bytes()).unwrap();

        assert_eq!(format!("{}", map), format!("{}\n", map_string));

        let map2: Map<(usize, usize), TestTile> = Map::new();
        assert_eq!(format!("{}", map2), "");
    }
}
