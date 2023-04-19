use std::vec::Vec;
use std::{dbg, vec};

use super::Set;
use crate::{
    concrete::{Prefix, PrefixRange},
    error::TestResult,
    Ipv4,
};

#[allow(clippy::missing_const_for_fn)]
fn assert_send<T: Send>(_: T) {}

#[allow(clippy::missing_const_for_fn)]
fn assert_sync<T: Sync>(_: T) {}

mod new_ipv4_prefix_set {
    use super::*;

    const fn setup() -> Set<Ipv4> {
        Set::new()
    }

    #[test]
    fn is_send() {
        let s = setup();
        assert_send(s);
    }

    #[test]
    fn is_sync() {
        let s = setup();
        assert_sync(s);
    }

    #[test]
    fn is_emtpy() {
        let s = setup();
        assert!(s.root.is_none());
    }

    #[test]
    fn contains_no_prefixes() -> TestResult {
        let s = setup();
        assert_eq!(s.prefixes().count(), 0);
        assert!(!s.contains("192.0.2.0/24".parse()?));
        assert!(!s.contains("192.0.0.0/22".parse()?));
        assert!(!s.contains("192.0.2.128/25".parse()?));
        assert!(!s.contains("192.0.4.0/24".parse()?));
        Ok(())
    }

    #[test]
    fn iter_over_prefixes_is_empty() {
        let s = setup();
        let c: Vec<_> = s.prefixes().collect();
        assert_eq!(Vec::<Prefix<Ipv4>>::new(), c);
    }

    mod with_a_prefix_added {
        use super::*;

        fn setup() -> Set<Ipv4> {
            let mut s = super::setup();
            let p = "192.0.2.0/24".parse::<Prefix<Ipv4>>().unwrap();
            s.insert(p).clone()
        }

        #[test]
        fn contains_one_prefix() {
            let s = setup();
            assert_eq!(s.prefixes().count(), 1);
        }

        #[test]
        fn contains_that_prefix() -> TestResult {
            let s = setup();
            assert!(s.contains("192.0.2.0/24".parse()?));
            Ok(())
        }

        #[test]
        fn does_not_contain_others() -> TestResult {
            let s = setup();
            assert!(!s.contains("192.0.0.0/22".parse()?));
            assert!(!s.contains("192.0.2.128/25".parse()?));
            assert!(!s.contains("192.0.4.0/24".parse()?));
            Ok(())
        }

        #[test]
        fn iter_over_prefixes_is_singleton() -> TestResult {
            let s = setup();
            let c: Vec<Prefix<Ipv4>> = s.prefixes().collect();
            assert_eq!(vec!["192.0.2.0/24".parse::<Prefix<Ipv4>>()?], c);
            Ok(())
        }

        mod and_removed {
            use super::*;

            fn setup() -> Set<Ipv4> {
                let mut s = super::setup();
                let p = "192.0.2.0/24".parse::<Prefix<Ipv4>>().unwrap();
                s.remove(p).clone()
            }

            #[test]
            fn is_emtpy() {
                let s = setup();
                assert!(s.root.is_none());
            }
        }

        mod with_another_prefix_added {
            use super::*;

            fn setup() -> Set<Ipv4> {
                let mut s = super::setup();
                let p = "192.0.0.0/22".parse::<Prefix<Ipv4>>().unwrap();
                s.insert(p).clone()
            }

            #[test]
            fn contains_two_prefixes() {
                let s = setup();
                assert_eq!(s.prefixes().count(), 2);
            }

            #[test]
            fn contains_both_prefixes() -> TestResult {
                let s = setup();
                assert!(s.contains("192.0.2.0/24".parse()?));
                assert!(s.contains("192.0.0.0/22".parse()?));
                Ok(())
            }

            #[test]
            fn iter_over_prefixes_is_len_two() -> TestResult {
                let s = setup();
                let c: Vec<_> = s.prefixes().collect();
                assert_eq!(
                    vec![
                        "192.0.0.0/22".parse::<Prefix<Ipv4>>()?,
                        "192.0.2.0/24".parse::<Prefix<Ipv4>>()?,
                    ],
                    c
                );
                Ok(())
            }

            mod and_a_range_removed {
                use super::*;

                fn setup() -> Set<Ipv4> {
                    let mut s = super::setup();
                    let r: PrefixRange<Ipv4> = "192.0.0.0/16,24,24".parse().unwrap();
                    s.remove(r).clone()
                }

                #[test]
                fn contains_one_prefix() {
                    let s = setup();
                    dbg!(&s);
                    assert_eq!(s.prefixes().count(), 1);
                }

                #[test]
                fn contains_the_remaining_prefix() -> TestResult {
                    let s = setup();
                    assert!(&s.contains("192.0.0.0/22".parse()?));
                    Ok(())
                }
            }

            mod with_a_third_prefix_added {
                use super::*;

                fn setup() -> Set<Ipv4> {
                    let mut s = super::setup();
                    let p: Prefix<Ipv4> = "192.0.3.0/24".parse().unwrap();
                    s.insert(p).clone()
                }

                #[test]
                fn contains_three_prefixes() {
                    let s = setup();
                    assert_eq!(s.prefixes().count(), 3);
                }

                #[test]
                fn contains_two_prefix_ranges() {
                    let s = setup();
                    assert_eq!(s.ranges().count(), 2);
                }

                #[test]
                fn contains_all_prefixes() -> TestResult {
                    let s = setup();
                    assert!(s.contains("192.0.2.0/24".parse()?));
                    assert!(s.contains("192.0.3.0/24".parse()?));
                    assert!(s.contains("192.0.0.0/22".parse()?));
                    Ok(())
                }

                #[test]
                fn iter_over_prefixes_is_len_three() -> TestResult {
                    let s = setup();
                    let c: Vec<_> = s.prefixes().collect();
                    assert_eq!(
                        vec![
                            "192.0.0.0/22".parse::<Prefix<Ipv4>>()?,
                            "192.0.2.0/24".parse::<Prefix<Ipv4>>()?,
                            "192.0.3.0/24".parse::<Prefix<Ipv4>>()?,
                        ],
                        c
                    );
                    Ok(())
                }

                mod and_a_range_removed {
                    use super::*;

                    fn setup() -> Set<Ipv4> {
                        let mut s = super::setup();
                        let r: PrefixRange<Ipv4> = "192.0.2.0/23,24,24".parse().unwrap();
                        s.remove(r).clone()
                    }

                    #[test]
                    fn contains_one_prefix() {
                        let s = setup();
                        assert_eq!(s.prefixes().count(), 1);
                    }

                    #[test]
                    fn contains_the_remaining_prefix() -> TestResult {
                        let s = setup();
                        assert!(s.contains("192.0.0.0/22".parse()?));
                        Ok(())
                    }
                }
            }
        }
    }
}
