use anyhow::Result;
use nom::bytes::complete::{take, take_until};
use nom::combinator::map_res;
use nom::sequence::terminated;
use nom::{IResult, Parser};
use nom_supreme::error::ErrorTree;
use nom_supreme::tag::complete::tag;
use std::str;
use std::time::Duration;
use thiserror::Error;

use crate::command::Command;
use crate::store::Expiry;

/// Terminating bytes between frames.
pub const CRLF: &str = "\r\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    Error(String),
    Simple(String),
    Bulk(String),
    Null,
    Array(Vec<Frame>),
}

#[derive(Debug, Error)]
pub enum Errors {
    #[error("error parsing input")]
    ParseError(String),
    #[error("frame is not recognized")]
    Unrecognised,
    #[error("input is incomplete")]
    Incomplete,
}

impl Frame {
    pub fn decode(input: &str) -> Result<Option<Frame>, Errors> {
        let parsed = decode_nom(input);
        match parsed {
            Ok((_, frame)) => Ok(frame),
            Err(e) => Err(Errors::ParseError(e.to_string())),
        }
    }

    fn unwrap_bulk(&self) -> String {
        match self {
            Frame::Bulk(str) => str.clone(),
            _ => panic!("not a bulk string"),
        }
    }

    pub fn to_command(&self) -> Result<Command> {
        let (command, args) = match self {
            Frame::Array(items) => Ok((
                items.first().unwrap().unwrap_bulk(),
                items.clone().into_iter().skip(1).collect::<Vec<Frame>>(),
            )),
            _ => Err(Errors::Unrecognised),
        }?;

        match command.to_uppercase().as_str() {
            "PING" => Ok(Command::Ping),
            "GET" => match args.get(0) {
                Some(Frame::Bulk(key)) => Ok(Command::Get(key.to_string())),
                _ => Err(Errors::Incomplete.into()),
            },
            "ECHO" => match args.get(0) {
                Some(Frame::Bulk(to_echo)) => Ok(Command::Echo(to_echo.to_string())),
                _ => Err(Errors::Incomplete.into()),
            },
            "SET" => match (args.get(0), args.get(1)) {
                (Some(Frame::Bulk(key)), Some(Frame::Bulk(value))) => {
                    let expiry = match (args.get(2), args.get(3)) {
                        (Some(Frame::Bulk(exp_type)), Some(Frame::Bulk(exp_amount))) => {
                            let amount: u64 = exp_amount.parse()?;
                            let duration = match exp_type.to_uppercase().as_ref() {
                                "PX" => Some(Duration::from_millis(amount)),
                                "EX" => Some(Duration::from_secs(amount)),
                                _ => None,
                            };

                            duration.map(Expiry::new)
                        }
                        _ => None,
                    };

                    Ok(Command::Set(key.to_string(), value.to_string(), expiry))
                }
                _ => Err(Errors::Incomplete.into()),
            },
            _ => Err(Errors::Unrecognised.into()),
        }
    }

    pub fn encode(&self) -> String {
        match self {
            Frame::Error(msg) => format!("-{}\r\n", msg.as_str()),
            Frame::Simple(s) => format!("+{}\r\n", s.as_str()),
            Frame::Bulk(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            Frame::Null => "$-1\r\n".to_string(),
            // The other cases are not required for the codecrafters challenge
            _ => unimplemented!(),
        }
    }
}

fn decode_nom(input: &str) -> IResult<&str, Option<Frame>, ErrorTree<&str>> {
    let (input, first) = take(1_usize)(input)?;
    match first {
        "*" => {
            let (input, len) = parse_len(input)?;

            let mut array = Vec::with_capacity(len);

            let mut input = input;
            for _ in 0..len {
                let (remaining, frame) = decode_nom(input)?;
                input = remaining;
                if let Some(f) = frame {
                    array.push(f);
                };
            }

            Ok((input, Some(Frame::Array(array))))
        }
        "+" => {
            let (input, simple) = parse_line(input)?;
            Ok((input, Some(Frame::Simple(simple.to_string()))))
        }
        "$" => {
            // We are not currently caring about the length of bulk strings, as we parse until the
            // CRLF
            let (input, _) = parse_len(input)?;
            let (input, bulk) = parse_line(input)?;
            Ok((input, Some(Frame::Bulk(bulk.to_string()))))
        }
        _ => unimplemented!(),
    }
}

fn parse_len(input: &str) -> IResult<&str, usize, ErrorTree<&str>> {
    map_res(parse_line, |s: &str| s.parse())(input)
}

fn parse_line(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    terminated(take_until(CRLF), tag(CRLF)).parse(input)
}

#[cfg(test)]
mod tests {
    use super::{decode_nom, Frame};

    #[test]
    fn test_parse_array_bulk() {
        let input = "*2\r\n$4\r\necho\r\n$5\r\nhello\r\n";
        let (input, decoded) = decode_nom(input).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            decoded,
            Some(Frame::Array(vec![
                Frame::Bulk("echo".to_string()),
                Frame::Bulk("hello".to_string())
            ]))
        )
    }
}
