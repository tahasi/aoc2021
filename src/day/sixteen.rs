use bitvec::{macros::internal::funty::Integral, prelude::*};
use std::{collections::VecDeque, path::PathBuf};

type Bits = BitSlice<u8, Msb0>;

use structopt::{self, StructOpt};

use super::read_all_text;

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(required(true), parse(from_os_str))]
    input: PathBuf,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let input = read_all_text(&self.input)?;
        let transmission = Transmission::parse(&input)?;

        println!(
            "transmission package version sum: {}",
            transmission.version_sum()
        );
        println!("transmission package decoded: {}", transmission.decode());
        Ok(())
    }
}

#[derive(Debug)]
struct Transmission {
    package: Package,
}

const VERSION_BIT_COUNT: usize = 3;
const TYPE_ID_BIT_COUNT: usize = 3;
const LITERAL_PACKET_DATA_BIT_COUNT: usize = 4;
const OPERATION_PACKET_DATA_BIT_COUNT: usize = 15;
const OPERATION_PACKET_DATA_COUNT: usize = 11;

fn split_first(
    bits: &mut &Bits,
    error_message: &'static str,
) -> Result<bool, ParseTransmissionError> {
    let (first, rest) = bits
        .split_first()
        .ok_or_else(|| ParseTransmissionError::new(error_message))?;
    *bits = rest;
    Ok(first == true)
}

fn split_at<'a>(
    bits: &mut &'a Bits,
    mid: usize,
    error_message: &'static str,
) -> Result<&'a Bits, ParseTransmissionError> {
    if bits.len() >= mid {
        let left: &Bits;
        let right: &Bits;
        unsafe {
            (left, right) = bits.split_at_unchecked(mid);
        }
        *bits = right;
        Ok(left)
    } else {
        let len = bits.len();
        Err(ParseTransmissionError::new(&format!(
            "{error_message}: len: {len}, mid: {mid}"
        )))
    }
}

fn split_at_as<I>(
    bits: &mut &Bits,
    mid: usize,
    error_message: &'static str,
) -> Result<I, ParseTransmissionError>
where
    I: Integral,
{
    Ok(split_at(bits, mid, error_message)?.load_be::<I>())
}

#[derive(Debug)]
enum Package {
    Literal {
        version: u8,
        value: u64,
    },
    Operator {
        version: u8,
        operation: Operation,
        packages: Vec<Package>,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum Operation {
    Sum = 0,
    Product = 1,
    Minimum = 2,
    Maximum = 3,
    GreaterThan = 5,
    LessThan = 6,
    EqualTo = 7,
}

impl TryFrom<u8> for Operation {
    type Error = ParseTransmissionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Operation::Sum),
            1 => Ok(Operation::Product),
            2 => Ok(Operation::Minimum),
            3 => Ok(Operation::Maximum),
            5 => Ok(Operation::GreaterThan),
            6 => Ok(Operation::LessThan),
            7 => Ok(Operation::EqualTo),
            _ => Err(ParseTransmissionError::new(&format!(
                "invalid operation value: {value}"
            ))),
        }
    }
}

impl Package {
    fn parse(bits: &mut &Bits) -> Result<Self, ParseTransmissionError> {
        let version = split_at_as::<u8>(
            bits,
            VERSION_BIT_COUNT,
            "insufficient bits for version header",
        )?;
        let type_id = split_at_as::<u8>(
            bits,
            TYPE_ID_BIT_COUNT,
            "insufficient bits for version header",
        )?;
        match type_id {
            4 => {
                let value = Package::parse_value(bits)?;
                Ok(Package::Literal { version, value })
            }
            _ => {
                let operation = Operation::try_from(type_id)?;
                let packages = Package::parse_sub_packages(bits)?;
                Ok(Package::Operator {
                    version,
                    operation,
                    packages,
                })
            }
        }
    }

    fn parse_value(bits: &mut &Bits) -> Result<u64, ParseTransmissionError> {
        let mut value: u64 = 0;
        let mut more_packets = true;
        while more_packets {
            more_packets = split_first(
                bits,
                "insufficient bits for literal packet identifier",
            )?;
            let packet_value = split_at_as::<u64>(
                bits,
                LITERAL_PACKET_DATA_BIT_COUNT,
                "insufficient bits for literal packet content",
            )?;
            value <<= 4;
            value += packet_value;
        }
        Ok(value)
    }

    fn parse_sub_packages(
        bits: &mut &Bits,
    ) -> Result<Vec<Package>, ParseTransmissionError> {
        let mut packages = vec![];
        let length_type_id = split_first(
            bits,
            "insufficient bits for packet length identifier",
        )?;
        if length_type_id {
            let mut packet_count = split_at_as::<u64>(
                bits,
                OPERATION_PACKET_DATA_COUNT,
                "insufficient bits for packet count",
            )?;
            while packet_count > 0 {
                packages.push(Package::parse(bits)?);
                packet_count -= 1;
            }
        } else {
            let packet_bit_count = split_at_as::<usize>(
                bits,
                OPERATION_PACKET_DATA_BIT_COUNT,
                "insufficient bits for packets bit count",
            )?;
            let mut packet_bits = split_at(
                bits,
                packet_bit_count,
                "insufficent bits for packet",
            )?;
            while !packet_bits.is_empty() {
                packages.push(Package::parse(&mut packet_bits)?);
            }
        }
        Ok(packages)
    }

    fn decode(&self) -> u64 {
        match self {
            Package::Literal { version: _, value } => *value as u64,
            Package::Operator {
                version: _,
                operation,
                packages,
            } => operation.execute(
                &packages
                    .iter()
                    .map(|package| package.decode())
                    .collect::<Vec<u64>>(),
            ),
        }
    }
}

impl Operation {
    fn execute(&self, values: &[u64]) -> u64 {
        let iter = values.iter();
        match self {
            Self::Sum => iter.sum(),
            Self::Product => iter.product(),
            Self::Minimum => {
                if let Some(min) = iter.min() {
                    *min
                } else {
                    0
                }
            }
            Self::Maximum => {
                if let Some(max) = iter.max() {
                    *max
                } else {
                    0
                }
            }
            Self::GreaterThan => {
                if values[0] > values[1] {
                    1
                } else {
                    0
                }
            }
            Self::LessThan => {
                if values[0] < values[1] {
                    1
                } else {
                    0
                }
            }
            Self::EqualTo => {
                if values[0] == values[1] {
                    1
                } else {
                    0
                }
            }
        }
    }
}

fn bitvec_from_str(
    input: &str,
) -> Result<BitVec<u8, Msb0>, ParseTransmissionError> {
    let mut bitvector = bitvec![u8, Msb0;];
    for hex_char in input.chars() {
        let hex_char_value = match hex_char {
            '0'..='9' => Ok(hex_char as u8 - b'0'),
            'A'..='F' => Ok(hex_char as u8 - b'A' + 10u8),
            invalid_char => Err(ParseTransmissionError::new(&format!(
                "input has an invalid character '{invalid_char}'"
            ))),
        }?;
        bitvector.extend(&hex_char_value.view_bits::<Msb0>()[4..])
    }
    Ok(bitvector)
}

impl Transmission {
    fn parse(input: &str) -> Result<Self, ParseTransmissionError> {
        let bitvector = bitvec_from_str(input.trim())?;
        let package = Package::parse(&mut &bitvector[..])?;
        Ok(Transmission { package })
    }

    fn version_sum(&self) -> u64 {
        let mut version_sum: u64 = 0;
        let mut pending_packages = VecDeque::from([&self.package]);
        while let Some(package) = pending_packages.pop_front() {
            version_sum += match package {
                Package::Literal { version, value: _ } => *version,
                Package::Operator {
                    version,
                    operation: _,
                    packages,
                } => {
                    for package in packages {
                        pending_packages.push_back(package);
                    }
                    *version
                }
            } as u64;
        }

        version_sum
    }

    fn decode(&self) -> u64 {
        self.package.decode()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse transmission from '{0}'")]
pub struct ParseTransmissionError(String);
impl ParseTransmissionError {
    fn new(text: &str) -> ParseTransmissionError {
        ParseTransmissionError(text.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use crate::day::sixteen::Operation;

    use super::{bitvec_from_str, Package, Transmission};

    #[test]
    pub fn test_d2fe28() {
        let bitvector = bitvec_from_str("D2FE28").expect("valid input");
        let package = Package::parse(&mut &bitvector[..]).expect("valid input");

        if let Package::Literal { version, value } = package {
            assert_eq!(6, version);
            assert_eq!(2021, value);
        } else {
            panic!("unexpected");
        }
    }

    #[test]
    pub fn test_38006f45291200() {
        let bitvector = bitvec_from_str("38006F45291200").expect("valid input");
        let package = Package::parse(&mut &bitvector[..]).expect("valid input");

        if let Package::Operator {
            version,
            operation,
            packages,
        } = package
        {
            assert_eq!(1, version);
            assert_eq!(Operation::LessThan, operation);
            assert_eq!(2, packages.len());
            if let Package::Literal { version: _, value } = packages[0] {
                assert_eq!(10, value);
            } else {
                panic!("expected operator first literal");
            }
            if let Package::Literal { version: _, value } = packages[1] {
                assert_eq!(20, value);
            } else {
                panic!("expected operator second literal");
            }
        } else {
            panic!("expected operator");
        }
    }

    #[test]
    pub fn test_ee00d40c823060() {
        let bitvector = bitvec_from_str("EE00D40C823060").expect("valid input");
        let package = Package::parse(&mut &bitvector[..]).expect("valid input");

        if let Package::Operator {
            version,
            operation,
            packages,
        } = package
        {
            assert_eq!(7, version);
            assert_eq!(Operation::Maximum, operation);
            assert_eq!(3, packages.len());
            if let Package::Literal { version: _, value } = packages[0] {
                assert_eq!(1, value);
            } else {
                panic!("expected operator first literal");
            }
            if let Package::Literal { version: _, value } = packages[1] {
                assert_eq!(2, value);
            } else {
                panic!("expected operator second literal");
            }
            if let Package::Literal { version: _, value } = packages[2] {
                assert_eq!(3, value);
            } else {
                panic!("expected operator third literal");
            }
        } else {
            panic!("expected operator");
        }
    }

    #[test]
    pub fn test_8a004a801a8002f478() {
        let transmission =
            Transmission::parse("8A004A801A8002F478").expect("valid input");

        assert_eq!(16, transmission.version_sum());
    }

    #[test]
    fn test_620080001611562c8802118e34() {
        let transmission = Transmission::parse("620080001611562C8802118E34")
            .expect("valid input");

        assert_eq!(12, transmission.version_sum());
    }

    #[test]
    fn test_c0015000016115a2e0802f182340() {
        let transmission = Transmission::parse("C0015000016115A2E0802F182340")
            .expect("valid input");

        assert_eq!(23, transmission.version_sum());
    }

    #[test]
    fn test_a0016c880162017c3686b18a3d4780() {
        let transmission =
            Transmission::parse("A0016C880162017C3686B18A3D4780")
                .expect("valid input");

        assert_eq!(31, transmission.version_sum());
    }

    #[test]
    fn test_c200b40a82() {
        let transmission =
            Transmission::parse("C200B40A82").expect("valid input");

        assert_eq!(3, transmission.decode());
    }

    #[test]
    fn test_04005ac33890() {
        let transmission =
            Transmission::parse("04005AC33890").expect("valid input");

        assert_eq!(54, transmission.decode());
    }

    #[test]
    fn test_880086c3e88112() {
        let transmission =
            Transmission::parse("880086C3E88112").expect("valid input");

        assert_eq!(7, transmission.decode());
    }

    #[test]
    fn test_ce00c43d881120() {
        let transmission =
            Transmission::parse("CE00C43D881120").expect("valid input");

        assert_eq!(9, transmission.decode());
    }

    #[test]
    fn test_d8005ac2a8f0() {
        let transmission =
            Transmission::parse("D8005AC2A8F0").expect("valid input");

        assert_eq!(1, transmission.decode());
    }

    #[test]
    fn test_f600bc2d8f() {
        let transmission =
            Transmission::parse("F600BC2D8F").expect("valid input");

        assert_eq!(0, transmission.decode());
    }

    #[test]
    fn test_9c005ac2f8f0() {
        let transmission =
            Transmission::parse("9C005AC2F8F0").expect("valid input");

        assert_eq!(0, transmission.decode());
    }

    #[test]
    fn test_9c0141080250320f1802104a08() {
        let transmission = Transmission::parse("9C0141080250320F1802104A08")
            .expect("valid input");

        assert_eq!(1, transmission.decode());
    }
}
