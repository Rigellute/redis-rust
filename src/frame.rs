use anyhow::Result;
use nom::bytes::complete::{take, take_until};
use nom::combinator::map_res;
use nom::sequence::terminated;
use nom::{IResult, Parser};
use nom_supreme::error::ErrorTree;
use nom_supreme::tag::complete::tag;
use std::str;
use thiserror::Error;

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
}

impl Frame {
    pub fn decode(input: &str) -> Result<Option<Frame>, Errors> {
        let parsed = decode_nom(input);
        match parsed {
            Ok((_, frame)) => Ok(frame),
            Err(e) => Err(Errors::ParseError(e.to_string())),
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
            unimplemented!()
        }
        "$" => {
            // We are not currently caring about the length of bulk strings, as we allocate the
            // line anyway
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
