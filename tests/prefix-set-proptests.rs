use std::collections::HashSet;

use ip::{traits::PrefixSet as _, AfiClass, Any, Ipv4, Ipv6, Prefix, PrefixSet};
use itertools::Itertools;
use proptest::{arbitrary::ParamsFor, prelude::*};

#[derive(Clone, Debug)]
struct TestPrefixSet<A: AfiClass> {
    ps: PrefixSet<A>,
    cs: HashSet<Prefix<A>>,
}

impl<A: AfiClass> FromIterator<Prefix<A>> for TestPrefixSet<A> {
    fn from_iter<T: IntoIterator<Item = Prefix<A>>>(iter: T) -> Self {
        let (ps_iter, cs_iter) = iter.into_iter().tee();
        Self {
            ps: ps_iter.collect(),
            cs: cs_iter.collect(),
        }
    }
}

impl<A: AfiClass> Arbitrary for TestPrefixSet<A>
where
    A: 'static,
    Prefix<A>: Arbitrary + 'static,
{
    type Parameters = ParamsFor<Vec<Prefix<A>>>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        any_with::<Vec<Prefix<A>>>(args)
            .prop_map(TestPrefixSet::from_iter)
            .boxed()
    }
}

macro_rules! property_tests {
    ( $( $mod:ident => $p:ty ),* $(,)? ) => {
        $(
            mod $mod {
                use super::*;

                proptest! {
                    #[test]
                    fn prefix_set_size(
                        s in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            dbg!(s.ps).prefixes().count(),
                            dbg!(s.cs).len()
                        );
                    }

                    #[test]
                    fn prefix_set_contains(
                        s in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert!(
                            s.cs.to_owned()
                                .into_iter()
                                .all(|p| s.ps.contains(p))
                        );
                    }

                    #[test]
                    fn prefix_set_contained(
                        s in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert!(
                            s.ps.prefixes()
                                .all(|p| s.cs.contains(&p))
                        );
                    }

                    #[test]
                    fn intersections_match(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            (s.ps & t.ps)
                                .prefixes()
                                .collect::<HashSet<_>>(),
                            &s.cs & &t.cs
                        )
                    }

                    #[test]
                    fn unions_match(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            (s.ps | t.ps)
                                .prefixes()
                                .collect::<HashSet<_>>(),
                            &s.cs | &t.cs
                        )
                    }

                    #[test]
                    fn differences_match(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            (s.ps - t.ps)
                                .prefixes()
                                .collect::<HashSet<_>>(),
                            &s.cs - &t.cs
                        )
                    }

                    #[test]
                    fn symmetric_differences_match(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        prop_assert_eq!(
                            (s.ps ^ t.ps)
                                .prefixes()
                                .collect::<HashSet<_>>(),
                            &s.cs ^ &t.cs
                        )
                    }

                    #[test]
                    fn intersection_le_sets(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        let intersection = s.ps.clone() & t.ps.clone();
                        prop_assert!(intersection <= s.ps);
                        prop_assert!(intersection <= t.ps);
                    }

                    #[test]
                    fn union_ge_sets(
                        s in any::<TestPrefixSet<$p>>(),
                        t in any::<TestPrefixSet<$p>>(),
                    ) {
                        let union = s.ps.clone() | t.ps.clone();
                        prop_assert!(union >= s.ps);
                        prop_assert!(union >= t.ps);
                    }
                }
            }
        )*
    }
}

property_tests! {
    ipv4 => Ipv4,
    ipv6 => Ipv6,
    any => Any,
}
