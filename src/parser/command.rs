// cue_sheet
// Copyright (C) 2017  Leonardo Schwarz <mail@leoschwarz.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use super::{FileFormat, TrackFlag, Time, Token, TrackType};
use std::str::FromStr;
use errors::Error;

/// The main grammar element of CUE sheets.
#[derive(Clone, Debug)]
pub enum Command {
    /// A 13-digit UPC/EAN code.
    Catalog(String),

    /// A path to a file containing CD-Text info.
    Cdtextfile(String),

    /// A path to a file containing audio data, and to which subsequent commands apply.
    File(String, FileFormat),

    /// Per-track subcode flag(s).
    Flags(Vec<TrackFlag>),

    /// Per-track index(es).
    Index(u32, Time),

    /// Per-track ISRC(s).
    Isrc(String),

    /// Per-disc or per-track performer name for CD-Text data.
    Performer(String),

    /// Amount of post-track silence to add.
    Postgap(Time),

    /// Amount of pre-track silence to add.
    Pregap(Time),

    /// A remark/comment to be ignored.
    /// (key,   value)
    Rem(String, Token),

    /// Per-disc or per-track songwriter name for CD-Text data.
    Songwriter(String),

    /// Per-disc or per-track title for CD-Text data.
    Title(String),

    /// Type of track to create, and to which subsequent commands apply.
    Track(u32, TrackType),
}

fn consume_token(tokens: &mut Vec<Token>) -> Result<Token, Error> {
    if tokens.len() == 0 {
        Err("No tokens left!".into())
    } else {
        Ok(tokens.remove(0))
    }
}

fn consume_time(tokens: &mut Vec<Token>) -> Result<Time, Error> {
    match consume_token(tokens)? {
        Token::Time(duration) => Ok(duration),
        t => Err(
            format!("Expected duration but found {:?} instead", t).into(),
        ),
    }
}

fn consume_number(tokens: &mut Vec<Token>) -> Result<u32, Error> {
    match consume_token(tokens)? {
        Token::Number(num) => Ok(num),
        t => Err(format!("Expeceted number but found {:?} instead", t).into()),
    }
}

fn consume_string(tokens: &mut Vec<Token>) -> Result<String, Error> {
    match consume_token(tokens)? {
        Token::String(s) => Ok(s),
        t => Err(format!("Expeceted string but found {:?} instead", t).into()),
    }
}

impl Command {
    pub(crate) fn consume(tokens: &mut Vec<Token>) -> Result<Command, Error> {
        let keyword = consume_string(tokens)?;
        match keyword.to_uppercase().as_str() {
            "CATALOG" => Ok(Command::Catalog(format!("{:013}", consume_number(tokens)?))),
            "CDTEXTFILE" => Ok(Command::Cdtextfile(consume_string(tokens)?)),
            "FILE" => Ok(Command::File(
                consume_string(tokens)?,
                consume_string(tokens)?.parse()?,
            )),
            "FLAGS" => {
                let mut flags = Vec::<TrackFlag>::new();

                while tokens.len() > 0 {
                    let token = tokens.remove(0);
                    let ok = match token {
                        Token::String(ref s) => {
                            match TrackFlag::from_str(s.as_str()) {
                                Ok(flag) => {
                                    flags.push(flag);
                                    true
                                }
                                Err(_) => false,
                            }
                        }
                        _ => false,
                    };

                    if !ok {
                        tokens.insert(0, token);
                        break;
                    }
                }

                if flags.len() == 0 {
                    Err(
                        "Encountered FLAGS command without succeeding TrackFlag".into(),
                    )
                } else {
                    Ok(Command::Flags(flags))
                }
            }
            "INDEX" => Ok(Command::Index(
                consume_number(tokens)?,
                consume_time(tokens)?,
            )),
            "ISRC" => Ok(Command::Isrc(consume_string(tokens)?)),
            "PERFORMER" => Ok(Command::Performer(consume_string(tokens)?)),
            "POSTGAP" => Ok(Command::Postgap(consume_time(tokens)?)),
            "PREGAP" => Ok(Command::Pregap(consume_time(tokens)?)),
            "REM" => Ok(Command::Rem(
                consume_string(tokens)?,
                consume_token(tokens)?,
            )),
            "SONGWRITER" => Ok(Command::Songwriter(consume_string(tokens)?)),
            "TITLE" => Ok(Command::Title(consume_string(tokens)?)),
            "TRACK" => Ok(Command::Track(
                consume_number(tokens)?,
                consume_string(tokens)?.parse()?,
            )),
            cmd => Err(format!("Invalid command: {:?}", cmd).into()),
        }
    }
}
