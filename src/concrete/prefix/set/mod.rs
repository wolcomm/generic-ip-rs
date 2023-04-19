use std::boxed::Box;
use std::mem;

use super::Prefix;
use crate::traits::Afi;

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
/// use ip::{Ipv6, Prefix, PrefixLength, PrefixRange};
/// use prefixset::{Error, PrefixSet};
///
/// fn main() -> Result<(), Error> {
///     // create a set by parsing a Vec<&str>
///     let set = vec!["2001:db8::/37", "2001:db8:f00::/37"]
///         .iter()
///         .map(|s| s.parse::<Prefix<Ipv6>>())
///         .collect::<Result<PrefixSet<_>, _>>()?;
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
/// # use ip::{Prefix, Ipv4};
/// # use prefixset::{Error, PrefixSet};
/// # fn main() -> Result<(), Error> {
/// let set = PrefixSet::new()
///     .insert("192.0.2.0/24".parse::<Prefix<Ipv4>>()?)
///     .to_owned();
/// assert_eq!(set.len(), 1);
/// #     Ok(())
/// # }
/// ```
///
/// [`bgpq3`]: https://github.com/snar/bgpq3
#[derive(Clone, Debug)]
pub struct Set<A: Afi> {
    root: Option<Box<Node<A>>>,
}

impl<A: Afi> Set<A> {
    /// Construct a new, empty [`PrefixSet<A>`].
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

    /// Insert a new `item` into `self`.
    ///
    /// `T` can be either a [`Prefix<A>`](ip::concrete::Prefix) or a
    /// [`PrefixRange<A>`](ip::concrete::PrefixRange).
    ///
    /// ``` rust
    /// # use ip::{PrefixRange, Ipv6};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let range: PrefixRange<Ipv6> = "2001:db8:f00::/48,64,64".parse()?;
    /// let set = PrefixSet::new().insert(range).to_owned();
    /// assert_eq!(set.len(), 1 << 16);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn insert<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Node<A>>,
    {
        self.insert_node(item.into().boxed()).aggregate()
    }

    /// Insert items into `self` from an iterator yielding either
    /// [`Prefix<A>`](ip::concrete::Prefix) or
    /// [`PrefixRange<A>`](ip::concrete::PrefixRange).
    ///
    /// Aggregation occurs after all items are inserted, making this far more
    /// efficient than calling [`PrefixSet::insert()`] repeatedly.
    ///
    /// ``` rust
    /// # use ip::{Ipv4, Prefix};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let prefixes: Vec<_> = vec!["192.0.2.0/26", "192.0.2.64/26"]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Prefix<Ipv4>>())
    ///     .collect::<Result<_, _>>()?;
    /// let set = PrefixSet::new().insert_from(prefixes).to_owned();
    /// assert_eq!(set.len(), 2);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn insert_from<I, T>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Node<A>>,
    {
        iter.into_iter()
            .fold(self, |set, item| set.insert_node(item.into().boxed()))
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
    /// `T` can be either a [`Prefix<A>`](ip::concrete::Prefix) or a
    /// [`PrefixRange<A>`](ip::concrete::PrefixRange).
    ///
    /// ``` rust
    /// # use ip::{Ipv6, Prefix};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = ["2001:db8:f00::/48", "2001:db8:baa::/48"]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Prefix<Ipv6>>())
    ///     .collect::<Result<PrefixSet<_>, _>>()?
    ///     .remove("2001:db8:f00::/48".parse::<Prefix<Ipv6>>()?)
    ///     .to_owned();
    /// assert_eq!(set.len(), 1);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn remove<T>(&mut self, item: T) -> &mut Self
    where
        T: Into<Node<A>>,
    {
        self.remove_node(item.into().boxed()).aggregate()
    }

    /// Remove items into `self` from an iterator yielding either
    /// [`Prefix<A>`](ip::concrete::Prefix) or
    /// [`PrefixRange<A>`](ip::concrete::PrefixRange).
    ///
    /// Aggregation occurs after all items are removed, making this far more
    /// efficient than calling [`PrefixSet::remove()`] repeatedly.
    ///
    /// ``` rust
    /// # use ip::{Ipv4, Prefix, PrefixRange};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let prefixes: Vec<_> = vec!["192.0.2.0/26", "192.0.2.64/26"]
    ///     .into_iter()
    ///     .map(|s| s.parse::<Prefix<Ipv4>>())
    ///     .collect::<Result<_, _>>()?;
    /// let mut set = PrefixSet::new()
    ///     .insert("192.0.2.0/24,26,26".parse::<PrefixRange<Ipv4>>()?)
    ///     .to_owned();
    /// assert_eq!(set.remove_from(prefixes).len(), 2);
    /// #     Ok(())
    /// # }
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

    fn aggregate(&mut self) -> &mut Self {
        if let Some(root) = mem::take(&mut self.root) {
            self.root = root.aggregate(None);
        }
        self
    }

    /// Test whether `prefix` is contained in `self`.
    ///
    /// ``` rust
    /// # use ip::{Ipv4, Prefix, PrefixRange};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = PrefixSet::new()
    ///     .insert("192.0.2.0/24,26,26".parse::<PrefixRange<Ipv4>>()?)
    ///     .to_owned();
    /// assert!(set.contains("192.0.2.128/26".parse()?));
    /// #     Ok(())
    /// # }
    /// ```
    pub fn contains(&self, prefix: Prefix<A>) -> bool {
        self.root
            .as_ref()
            .map_or(false, |root| root.search(&prefix.into()).is_some())
    }

    /// Get the number of prefixes in `self`.
    ///
    /// ``` rust
    /// # use ip::{Ipv4, PrefixRange};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = PrefixSet::new()
    ///     .insert("192.0.2.0/24,26,26".parse::<PrefixRange<Ipv4>>()?)
    ///     .to_owned();
    /// assert_eq!(set.len(), 4);
    /// #     Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.prefixes().count()
    }

    /// Test whether `self` is empty.
    ///
    /// ``` rust
    /// # use ip::Ipv4;
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// assert!(PrefixSet::<Ipv4>::new().is_empty());
    /// #     Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ranges().count() == 0
    }

    /// Clear the contents of `self`
    ///
    /// ``` rust
    /// # use ip::{Ipv6, Prefix};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let mut set = PrefixSet::new()
    ///     .insert("2001:db8::/32".parse::<Prefix<Ipv6>>()?)
    ///     .to_owned();
    /// assert!(!set.is_empty());
    /// set.clear();
    /// assert!(set.is_empty());
    /// #     Ok(())
    /// # }
    /// ```
    pub fn clear(&mut self) {
        self.root = None;
    }

    /// Get an iterator over the [`PrefixRange<A>`](ip::concrete::PrefixRange)s
    /// contained in `self`.
    ///
    /// ``` rust
    /// # use ip::{Ipv4, Prefix};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = PrefixSet::new()
    ///     .insert("192.0.2.0/25".parse::<Prefix<Ipv4>>()?)
    ///     .insert("192.0.2.128/25".parse::<Prefix<Ipv4>>()?)
    ///     .to_owned();
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some("192.0.2.0/24,25,25".parse()?));
    /// assert_eq!(ranges.next(), None);
    /// #     Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn ranges(&self) -> Ranges<'_, A> {
        self.into()
    }

    /// Get an iterator over the [`Prefix<A>`](ip::concrete::Prefix)s
    /// contained in `self`.
    ///
    /// ``` rust
    /// # use ip::{Ipv4, Prefix};
    /// # use prefixset::{Error, PrefixSet};
    /// # fn main() -> Result<(), Error> {
    /// let set = PrefixSet::new()
    ///     .insert("192.0.2.0/25".parse::<Prefix<Ipv4>>()?)
    ///     .insert("192.0.2.128/25".parse::<Prefix<Ipv4>>()?)
    ///     .to_owned();
    /// let mut prefixes = set.prefixes();
    /// assert_eq!(prefixes.next(), Some("192.0.2.0/25".parse()?));
    /// assert_eq!(prefixes.next(), Some("192.0.2.128/25".parse()?));
    /// assert_eq!(prefixes.next(), None);
    /// #     Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn prefixes(&self) -> Prefixes<'_, A> {
        self.into()
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
