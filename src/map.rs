use std::collections::HashMap;

use snafu::{ResultExt, Snafu};
use std::io::{BufRead, BufReader};

pub trait MapTile: std::fmt::Display + Sized + Clone {
    fn from_char(c: char) -> Option<Self>;
}

pub trait MapCoordinate: Default + Eq + std::hash::Hash + std::fmt::Debug + Clone {
    fn get_extent<'a>(keys: impl Iterator<Item = Self>) -> Self;
}

impl MapCoordinate for usize {
    fn get_extent<'a>(keys: impl Iterator<Item = Self>) -> Self {
        keys.max().map(|s| s + 1).unwrap_or(0)
    }
}

impl MapCoordinate for (usize, usize) {
    fn get_extent<'a>(keys: impl Iterator<Item = Self>) -> Self {
        let mut imax = 0;
        let mut jmax = 0;
        for k in keys {
            imax = imax.max(k.0 + 1);
            jmax = jmax.max(k.1 + 1);
        }

        return (imax, jmax);
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
    pub fixed_extent: Option<C>,
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
    pub fn get_extent(&self) -> C {
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

////// Code for 1D maps
impl<T: MapTile> Map<usize, T> {
    pub fn read<R: std::io::Read>(reader: &mut R) -> MapResult<Self> {
        let mut data = HashMap::new();

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).context(Io)?;

        let s = String::from_utf8_lossy(&buf);

        for (i, c) in s.chars().enumerate() {
            if let Some(t) = T::from_char(c) {
                data.insert(i, t);
            }
        }

        Ok(Map {
            data,
            fixed_extent: None,
        })
    }

    pub fn to_vecs(&self) -> Vec<Option<T>> {
        let width = self.get_extent();
        (0..width).map(|i| self.data.get(&i).cloned()).collect()
    }
}

impl<T: MapTile> std::fmt::Display for Map<usize, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        if self.data.is_empty() {
            return Ok(());
        }

        let width = self.get_extent();
        for i in 0..width {
            match self.data.get(&i) {
                Some(t) => t.fmt(f),
                None => write!(f, " "),
            }?;
        }

        Ok(())
    }
}

////// Code for 2D maps
impl<T: MapTile> Map<(usize, usize), T> {
    pub fn read<R: std::io::Read>(reader: &mut R) -> MapResult<Self> {
        let mut data = HashMap::new();

        let buf_reader = BufReader::new(reader);
        for (i, line) in buf_reader.lines().enumerate() {
            for (j, c) in line.context(Io)?.chars().enumerate() {
                if let Some(t) = T::from_char(c) {
                    data.insert((i, j), t);
                }
            }
        }

        Ok(Map {
            data,
            fixed_extent: None,
        })
    }

    pub fn to_vecs(&self) -> Vec<Vec<Option<T>>> {
        let (height, width) = self.get_extent();

        (0..height)
            .map(|i| {
                (0..width)
                    .map(|j| self.data.get(&(i, j)).cloned())
                    .collect()
            })
            .collect()
    }
}

impl<T: MapTile> std::fmt::Display for Map<(usize, usize), T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        if self.data.is_empty() {
            return Ok(());
        }

        let (height, width) = self.get_extent();

        for i in 0..height {
            for j in 0..width {
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

impl<T: MapTile> std::str::FromStr for Map<(usize, usize), T> {
    type Err = MapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Map::<(usize, usize), T>::read(&mut s.as_bytes())
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
    fn test_1d_parsing() {
        let map_string = "ab d f";
        let map = Map::<usize, TestTile>::read(&mut map_string.as_bytes()).unwrap();

        assert_eq!(map.get_extent(), 6);

        assert_eq!(
            map.to_vecs(),
            vec![
                Some(TestTile('a')),
                Some(TestTile('b')),
                None,
                Some(TestTile('d')),
                None,
                Some(TestTile('f'))
            ],
        )
    }

    #[test]
    fn test_1d_editing() {
        let mut map: Map<usize, TestTile> = Map::new();
        assert_eq!(map.get_extent(), 0);

        assert_eq!(map.get(&1), None);
        map.set(1, TestTile('a'));
        assert_eq!(map.get(&1), Some(&TestTile('a')));
        assert_eq!(map.get_extent(), 2);

        map.set(4, TestTile('c'));
        assert_eq!(map.get_extent(), 5);

        map.set(8, TestTile('d'));
        assert_eq!(map.get_extent(), 9);

        map.remove(&8);
        assert_eq!(map.get_extent(), 5);

        assert_eq!(
            map.to_vecs(),
            vec![None, Some(TestTile('a')), None, None, Some(TestTile('c'))]
        )
    }

    #[test]
    fn test_1d_display() {
        let map_string = "ab d f";
        let map = Map::<usize, TestTile>::read(&mut map_string.as_bytes()).unwrap();

        assert_eq!(format!("{}", map), format!("{}", map_string));

        let map2: Map<usize, TestTile> = Map::new();
        assert_eq!(format!("{}", map2), "");
    }

    #[test]
    fn test_2d_parsing() {
        let map_string = "ab \nd e";
        let map = Map::<(usize, usize), TestTile>::read(&mut map_string.as_bytes()).unwrap();

        assert_eq!(map.get_extent(), (2, 3));

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
        assert_eq!(map.get_extent(), (5, 3));

        map.set((8, 8), TestTile('d'));
        assert_eq!(map.get_extent(), (9, 9));

        map.remove(&(8, 8));
        assert_eq!(map.get_extent(), (5, 3));

        assert_eq!(
            map.to_vecs(),
            vec![
                vec![None, None, None],
                vec![None, None, Some(TestTile('a'))],
                vec![None, None, None],
                vec![None, None, None],
                vec![None, Some(TestTile('c')), None],
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
