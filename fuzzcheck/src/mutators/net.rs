use std::net::{IpAddr, SocketAddr};

use crate::{DefaultMutator, ExtendedMutator, MutatorValueConverter, MutatorWrapper};

use super::integer_within_range::U32WithinRangeMutator;

fn ipv4_to_u32(ip: IpAddr) -> u32 {
    match ip {
        IpAddr::V4(addr) => {
            let [a, b, c, d] = addr.octets();
            let (a, b, c, d) = (a as u32, b as u32, c as u32, d as u32);
            a * 256 * 256 * 256 + b * 256 * 256 + c * 256 + d
        }
        IpAddr::V6(_) => panic!("expected ipv4, found ipv6"),
    }
}

fn ipv4_from_u32(id: u32) -> IpAddr {
    let ip = [
        ((id / 256 / 256 / 256) % 256) as u8,
        ((id / 256 / 256) % 256) as u8,
        ((id / 256) % 256) as u8,
        (id % 256) as u8,
    ];

    ip.into()
}

fn addr_from_u32(id: u32) -> SocketAddr {
    (ipv4_from_u32(id), 12345).into()
}

impl DefaultMutator for IpAddr {
    type Mutator = IpAddrMutator;

    fn default_mutator() -> Self::Mutator {
        IpAddrMutator::new(100)
    }
}

impl DefaultMutator for SocketAddr {
    type Mutator = SocketAddrMutator;

    fn default_mutator() -> Self::Mutator {
        SocketAddrMutator::new(100)
    }
}

pub struct IpAddrU32Converter {}

impl IpAddrU32Converter {
    pub fn new() -> Self {
        Self {}
    }
}

impl MutatorValueConverter for IpAddrU32Converter {
    type InnerValue = u32;
    type Value = IpAddr;

    fn from_inner_value_ref(&self, inner_value: &Self::InnerValue) -> Self::Value {
        ipv4_from_u32(*inner_value)
    }

    fn to_inner_value_ref(&self, value: &Self::Value) -> Self::InnerValue {
        ipv4_to_u32(*value)
    }
}

pub type IpAddrMutatorInner = ExtendedMutator<IpAddr, u32, U32WithinRangeMutator, IpAddrU32Converter>;

pub struct IpAddrMutator {
    inner: IpAddrMutatorInner,
}

impl IpAddrMutator {
    pub fn new(limit: u32) -> Self {
        Self {
            inner: ExtendedMutator::new(U32WithinRangeMutator::new(1..=limit), IpAddrU32Converter::new()),
        }
    }
}

impl MutatorWrapper for IpAddrMutator {
    type Wrapped = IpAddrMutatorInner;

    fn wrapped_mutator(&self) -> &Self::Wrapped {
        &self.inner
    }
}

pub struct SocketAddrU32Converter {}

impl SocketAddrU32Converter {
    pub fn new() -> Self {
        Self {}
    }
}

impl MutatorValueConverter for SocketAddrU32Converter {
    type InnerValue = u32;
    type Value = SocketAddr;

    fn from_inner_value_ref(&self, inner_value: &Self::InnerValue) -> Self::Value {
        addr_from_u32(*inner_value)
    }

    fn to_inner_value_ref(&self, value: &Self::Value) -> Self::InnerValue {
        ipv4_to_u32(value.ip())
    }
}

pub type SocketAddrMutatorInner = ExtendedMutator<SocketAddr, u32, U32WithinRangeMutator, SocketAddrU32Converter>;

pub struct SocketAddrMutator {
    inner: SocketAddrMutatorInner,
}

impl SocketAddrMutator {
    pub fn new(limit: u32) -> Self {
        Self {
            inner: ExtendedMutator::new(U32WithinRangeMutator::new(1..=limit), SocketAddrU32Converter::new()),
        }
    }
}

impl MutatorWrapper for SocketAddrMutator {
    type Wrapped = SocketAddrMutatorInner;

    fn wrapped_mutator(&self) -> &Self::Wrapped {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutators::testing_utilities::test_mutator;

    #[test]
    fn test_ipaddr_mutator() {
        test_mutator(IpAddrMutator::new(100), 99.0, 99.0, true, 0, 99);
    }

    #[test]
    fn test_socketaddr_mutator() {
        test_mutator(SocketAddrMutator::new(100), 99.0, 99.0, true, 0, 99);
    }
}
