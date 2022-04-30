pub trait Address {
    fn is_broadcast(&self) -> bool;
    fn is_link_local(&self) -> bool;
    fn is_private(&self) -> bool;
    fn is_reserved(&self) -> bool;
    fn is_shared(&self) -> bool;
    fn is_thisnet(&self) -> bool;
    fn is_benchmarking(&self) -> bool;
    fn is_documentation(&self) -> bool;
    fn is_global(&self) -> bool;
    fn is_loopback(&self) -> bool;
    fn is_multicast(&self) -> bool;
    fn is_unspecified(&self) -> bool;
    fn is_unique_local(&self) -> bool;
    fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }
    fn is_unicast_global(&self) -> bool {
        self.is_unicast() && self.is_global()
    }
}
