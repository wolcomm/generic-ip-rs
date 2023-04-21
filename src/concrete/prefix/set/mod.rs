use std::boxed::Box;
use std::mem;

use super::{Prefix, Range};
use crate::traits::{self, Afi};

mod iter;
use self::iter::{Prefixes, Ranges};

mod node;
use self::node::Node;

mod ops;

/// A collection of IP prefixes, providing fast insertion and iteration,
/// and set-theorectic arithmetic.
///
/// This is a Rust implementation derived in large part from the internal
/// data-structure used in the widely used [`bgpq3`] tool by Alexandre Snarskii,
/// packaged as a library, and with set-theoretic operations added.
///
/// # Examples
///
/// ``` rust
/// use ip::{traits::PrefixSet as _, Error, Ipv6, Prefix, PrefixLength, PrefixRange, PrefixSet};
///
/// fn main() -> Result<(), Error> {
///     // create a set by parsing a Vec<&str>
///     let set: PrefixSet<Ipv6> = ["2001:db8::/37", "2001:db8:f00::/37"]
///         .into_iter()
///         .map(|s| s.parse::<Prefix<Ipv6>>())
///         .collect::<Result<_, _>>()?;
///
///     // create a range by parsing a &str and providing the lower
///     // and upper prefix lenth bounds
///     let length = PrefixLength::<Ipv6>::from_primitive(37)?;
///     let range = PrefixRange::<Ipv6>::new("2001:db8::/36".parse()?, length..=length)?;
///
///     assert_eq!(set.ranges().collect::<Vec<_>>(), vec![range]);
///     Ok(())
/// }
/// ```
///
/// Most mutating methods return `&mut Self` for easy chaining, e.g.:
///
/// ``` rust
/// # use ip::{traits::PrefixSet as _, Error, Ipv4, Prefix, PrefixSet};
/// let set = PrefixSet::<Ipv4>::new()
///     .insert("192.0.2.0/24".parse::<Prefix<Ipv4>>()?)
///     .to_owned();
/// assert_eq!(set.len(), 1);
/// # Ok::<_, Error>(())
/// ```
///
/// [`bgpq3`]: https://github.com/snar/bgpq3
#[derive(Clone, Debug)]
pub struct Set<A: Afi> {
    root: Option<Box<Node<A>>>,
}

impl<A: Afi> Set<A> {
    /// Construct a new, empty [`PrefixSet<A>`][Self].
    #[must_use]
    pub const fn new() -> Self {
        Self::new_with_root(None)
    }

    const fn new_with_root(root: Option<Box<Node<A>>>) -> Self {
        Self { root }
    }

    fn insert_node(&mut self, new: Box<Node<A>>) -> &mut Self {
        match mem::take(&mut self.root) {
            Some(root) => {
                self.root = Some(root.add(new));
            }
            None => {
                self.root = Some(new);
            }
        };
        self
    }

    pub(crate) fn insert_only<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Node<A>>,
    {
        self.insert_node(item.into().boxed())
    }

    /// Insert a new `item` into `self`.
    ///
    /// `T` can be either a [`Prefix<A>`](crate::concrete::Prefix) or a
    /// [`PrefixRange<A>`](crate::concrete::PrefixRange).
    ///
    /// ``` rust
    /// # use ip::{traits::PrefixSet as _, Error, Ipv6, PrefixRange, PrefixSet};
    /// let range: PrefixRange<Ipv6> = "2001:db8:f00::/48,64,64".parse()?;
    /// let set = PrefixSet::<Ipv6>::new().insert(range).to_owned();
    /// assert_eq!(set.len(), 1 << 16);
    /// # Ok::<_, Error>(())
    /// ```
    pub fn insert<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Node<A>>,
    {
        self.insert_only(item).aggregate()
    }

    /// Insert items into `self` from an iterator yielding either
    /// [`Prefix<A>`](crate::concrete::Prefix) or
    /// [`PrefixRange<A>`](crate::concrete::PrefixRange).
    ///
    /// Aggregation occurs after all items are inserted, making this far more
    /// efficient than calling [`PrefixSet::insert()`][Self::insert] repeatedly.
    ///
    /// ``` rust
    /// # use ip::{traits::PrefixSet as _, Error, Ipv4, Prefix, PrefixSet};
    /// let prefixes: Vec<_> = ["192.0.2.0/26", "192.0.2.64/26"]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Prefix<Ipv4>>())
    ///     .collect::<Result<_, _>>()?;
    /// let set = PrefixSet::<Ipv4>::new().insert_from(prefixes).to_owned();
    /// assert_eq!(set.len(), 2);
    /// # Ok::<_, Error>(())
    /// ```
    pub fn insert_from<I, T>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Node<A>>,
    {
        iter.into_iter()
            .fold(self, |set, item| set.insert_only(item))
            .aggregate()
    }

    fn remove_node(&mut self, mut old: Box<Node<A>>) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = Some(root.remove(&mut old));
        };
        self
    }

    /// Remove an `item` from `self`.
    ///
    /// `T` can be either a [`Prefix<A>`](crate::concrete::Prefix) or a
    /// [`PrefixRange<A>`](crate::concrete::PrefixRange).
    ///
    /// ``` rust
    /// # use ip::{traits::PrefixSet as _, Error, Ipv6, Prefix, PrefixSet};
    /// let set = ["2001:db8:f00::/48", "2001:db8:baa::/48"]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Prefix<Ipv6>>())
    ///     .collect::<Result<PrefixSet<Ipv6>, _>>()?
    ///     .remove("2001:db8:f00::/48".parse::<Prefix<Ipv6>>()?)
    ///     .to_owned();
    /// assert_eq!(set.len(), 1);
    /// # Ok::<_, Error>(())
    /// ```
    pub fn remove<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Node<A>>,
    {
        self.remove_node(item.into().boxed()).aggregate()
    }

    /// Remove items from `self` from an iterator yielding either
    /// [`Prefix<A>`](crate::concrete::Prefix) or
    /// [`PrefixRange<A>`](crate::concrete::PrefixRange).
    ///
    /// Aggregation occurs after all items are removed, making this far more
    /// efficient than calling [`PrefixSet::remove()`][Self::remove] repeatedly.
    ///
    /// ``` rust
    /// # use ip::{traits::PrefixSet as _, Error, Ipv4, Prefix, PrefixRange, PrefixSet};
    /// let prefixes: Vec<_> = vec!["192.0.2.0/26", "192.0.2.64/26"]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Prefix<Ipv4>>())
    ///     .collect::<Result<_, _>>()?;
    /// let mut set = PrefixSet::<Ipv4>::new()
    ///     .insert("192.0.2.0/24,26,26".parse::<PrefixRange<Ipv4>>()?)
    ///     .to_owned();
    /// assert_eq!(set.remove_from(prefixes).len(), 2);
    /// # Ok::<_, Error>(())
    /// ```
    pub fn remove_from<I, T>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Node<A>>,
    {
        iter.into_iter()
            .fold(self, |set, item| set.remove_node(item.into().boxed()))
            .aggregate()
    }

    pub(crate) fn aggregate(&mut self) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = root.aggregate(None);
        }
        self
    }

    /// Clear the contents of `self`
    ///
    /// ``` rust
    /// # use ip::{traits::PrefixSet as _, Error, Ipv6, Prefix, PrefixSet};
    /// let mut set = PrefixSet::<Ipv6>::new()
    ///     .insert("2001:db8::/32".parse::<Prefix<Ipv6>>()?)
    ///     .to_owned();
    /// assert!(!set.is_empty());
    /// set.clear();
    /// assert!(set.is_empty());
    /// # Ok::<_, Error>(())
    /// ```
    pub fn clear(&mut self) {
        self.root = None;
    }
}

impl<'a, A: Afi> traits::PrefixSet<'a> for Set<A> {
    type Prefix = Prefix<A>;
    type Range = Range<A>;
    type Prefixes = Prefixes<'a, A>;
    type Ranges = Ranges<'a, A>;

    fn prefixes(&'a self) -> Self::Prefixes {
        self.into()
    }

    fn ranges(&'a self) -> Self::Ranges {
        self.into()
    }

    fn contains(&self, prefix: Self::Prefix) -> bool {
        self.root
            .as_ref()
            .map_or(false, |root| root.search(&prefix.into()).is_some())
    }
}

impl<A: Afi> Default for Set<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Afi, U> Extend<U> for Set<A>
where
    U: Into<Node<A>>,
{
    #[allow(unused_results)]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = U>,
    {
        self.insert_from(iter);
    }
}

impl<A: Afi, T> FromIterator<T> for Set<A>
where
    T: Into<Node<A>>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self::new().insert_from(iter).clone()
    }
}

#[cfg(test)]
mod tests;
