use std::collections::HashMap;

use snafu::{ResultExt, Snafu};
use std::io::{BufRead, BufReader};

pub trait MapTile: std::fmt::Display + Sized + Clone {
    fn from_char(c: char) -> Option<Self>;
}

#[derive(Debug, Snafu)]
pub enum MapError {
    #[snafu(display("I/O error: {}", source))]
    Io { source: std::io::Error },
}

type MapResult<T> = std::result::Result<T, MapError>;

#[derive(Default, Debug, Clone)]
pub struct Map<T> {
    data: HashMap<(usize, usize), T>,
}

impl<T: MapTile> Map<T> {
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

        Ok(Map { data })
    }

    pub fn to_vecs(&self) -> Vec<Vec<Option<T>>> {
        let (imin, imax, jmin, jmax) = self.get_extent();

        (imin..=imax)
            .map(|i| (jmin..=jmax).map(|j| self.get(&(i, j)).cloned()).collect())
            .collect()
    }
}

impl<T> Map<T> {
    pub fn get_extent(&self) -> (usize, usize, usize, usize) {
        let (mut imin, mut jmin) = self.data.keys().next().unwrap();
        let (mut imax, mut jmax) = (imin, jmin);
        for (i, j) in self.data.keys() {
            if *i < imin {
                imin = *i;
            }
            if *j < jmin {
                jmin = *j;
            }
            if *i > imax {
                imax = *i;
            }
            if *j > jmax {
                jmax = *j;
            }
        }

        (imin, imax, jmin, jmax)
    }
}

impl<T> std::ops::Deref for Map<T> {
    type Target = HashMap<(usize, usize), T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for Map<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: MapTile> std::fmt::Display for Map<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        if self.data.is_empty() {
            return Ok(());
        }

        let (imin, imax, jmin, jmax) = self.get_extent();

        for i in imin..=imax {
            for j in jmin..=jmax {
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
    fn test_it_works() {
        let map_string = "ab \nd e";
        let map: Map<TestTile> = Map::read(&mut map_string.as_bytes()).unwrap();

        assert_eq!(
            map.to_vecs(),
            vec![
                vec![Some(TestTile('a')), Some(TestTile('b')), None],
                vec![Some(TestTile('d')), None, Some(TestTile('e'))],
            ]
        )
    }
}
