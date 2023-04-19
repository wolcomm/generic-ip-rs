use std::fs::File;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::str::FromStr;

pub struct DataSet<T> {
    name: &'static str,
    prefixes: usize,
    ranges: usize,
    _data: PhantomData<T>,
}

pub const fn data_set<T>(name: &'static str, prefixes: usize, ranges: usize) -> DataSet<T> {
    DataSet {
        name,
        prefixes,
        ranges,
        _data: PhantomData,
    }
}

#[macro_export]
macro_rules! data_sets {
    ( $( name = $name:literal, prefixes = $prefixes:literal, ranges = $ranges:literal );* $(;)? ) => {
        &[
            $(
                $crate::data_set($name, $prefixes, $ranges)
            ),*
        ]
    };
}

impl<T> DataSet<T> {
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn prefixes(&self) -> usize {
        self.prefixes
    }
    pub fn ranges(&self) -> usize {
        self.ranges
    }
}

impl<T> DataSet<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    pub fn read(&self) -> Vec<T> {
        let path = format!("./test-data/{}.txt", self.name());
        let file = File::open(path).unwrap();
        BufReader::new(file)
            .lines()
            .into_iter()
            .map(|line| line.unwrap().parse::<T>().unwrap())
            .collect()
    }
}

impl<T> PartialEq for DataSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}
