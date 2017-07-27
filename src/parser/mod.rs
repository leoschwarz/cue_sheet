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

//! Parsing of cue sheets. Also contains some data types.

use errors::Error;
use std::str::FromStr;
use std::fmt;
use std::ops::Sub;

mod tokenization;
use self::tokenization::tokenize;
pub use self::tokenization::Token;

mod command;
pub use self::command::Command;

/// Number of audio frames per second in cue sheets.
const FPS: i64 = 75;

/// Time representation of the format `mm:ss:ff`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Time {
    /// Number of minutes.
    pub mins: i32,

    /// Number of seconds.
    pub secs: i8,

    /// Number of frames.
    ///
    /// Cue sheets assume a fixed number of 75 frames per second, if your audio file has a
    /// different rate, you have to do the math yourself.
    pub frames: i8,
}

impl Time {
    /// Create a new instance with the specified field values.
    pub fn new(mins: i32, secs: i8, frames: i8) -> Time {
        Time {
            mins: mins,
            secs: secs,
            frames: frames,
        }
    }

    /// Return a String consisting of only the mins and secs components.
    pub fn to_string_2(&self) -> String {
        format!("{:02}:{:02}", self.mins, self.secs)
    }

    fn to_frames(&self) -> i64 {
        ((self.mins * 60) as i64 + self.secs as i64) * FPS + self.frames as i64
    }

    fn from_frames(from: i64) -> Time {
        let frames = from % FPS;
        let secs_all = from / FPS;
        let secs = secs_all % 60;
        let mins = secs_all / 60;

        Time {
            mins: mins as i32,
            secs: secs as i8,
            frames: frames as i8,
        }
    }
}

impl FromStr for Time {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 8 {
            return Err("Time was not 8 chars long.".into());
        }

        if s.chars().nth(2).unwrap() != ':' || s.chars().nth(5).unwrap() != ':' {
            return Err("Time was not properly formatted.".into());
        }

        Ok(Time {
            mins: s[0..2].parse()?,
            secs: s[3..5].parse()?,
            frames: s[6..8].parse()?,
        })
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.mins, self.secs, self.frames)
    }
}

impl Sub for Time {
    type Output = Time;

    fn sub(self, rhs: Time) -> Self::Output {
        let left = self.to_frames();
        let right = rhs.to_frames();

        let diff = left - right;
        Time::from_frames(diff)
    }
}

/// Describes the file format of an audio file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileFormat {
    /// Also includes other lossless formats.
    Wave,

    /// An MP3 audio file.
    Mp3,

    /// An AIFF audio file.
    Aiff,

    /// Little-endian binary raw data file.
    Binary,

    /// Big-endian binary raw data file.
    Motorola,
}

impl FromStr for FileFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WAVE" => Ok(FileFormat::Wave),
            "MP3" => Ok(FileFormat::Mp3),
            "AIFF" => Ok(FileFormat::Aiff),
            "Binary" => Ok(FileFormat::Binary),
            "Motorola" => Ok(FileFormat::Motorola),
            _ => Err(format!("Invalid FileFormat: {:?}", s).into()),
        }
    }
}

/// Additional flags a Track can have.
#[derive(Clone, Debug)]
pub enum TrackFlag {
    /// Digital Copy Permitted
    Dcp,

    /// 4 Channel audio (4CH)
    FourChannel,

    /// PRE-emphasis enabled (audio tracks only)
    Pre,

    /// Serial Copy Management System
    Scms,
}

impl FromStr for TrackFlag {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DCP" => Ok(TrackFlag::Dcp),
            "4CH" => Ok(TrackFlag::FourChannel),
            "PRE" => Ok(TrackFlag::Pre),
            "SCMS" => Ok(TrackFlag::Scms),
            s => Err(format!("invalid TrackFlag: {:?}", s).into()),
        }
    }
}

/// Describes the type of tracks on the media.
///
/// Most of the times for music this will be just `Audio`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrackType {
    /// Audio/Music (2352 — 588 samples)
    Audio,

    /// Karaoke CD+G (2448)
    Cdg,

    /// * (1, 2048): CD-ROM Mode 1 Data (cooked)
    /// * (1, 2352): CD-ROM Mode 1 Data (raw)
    /// * (2, 2048): CD-ROM XA Mode 2 Data (form 1) *
    /// * (2, 2324): CD-ROM XA Mode 2 Data (form 2) *
    /// * (2, 2336): CD-ROM XA Mode 2 Data (form mix)
    /// * (2, 2352): CD-ROM XA Mode 2 Data (raw)
    Mode(u8, u16),

    /// * 2336: CDI Mode 2 Data
    /// * 2352: CDI Mode 2 Data
    Cdi(u16),
}

impl FromStr for TrackType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AUDIO" => Ok(TrackType::Audio),
            "CDG" => Ok(TrackType::Cdg),
            "MODE1/2048" => Ok(TrackType::Mode(1, 2048)),
            "MODE1/2352" => Ok(TrackType::Mode(1, 2352)),
            "MODE2/2048" => Ok(TrackType::Mode(1, 2048)),
            "MODE2/2324" => Ok(TrackType::Mode(1, 2324)),
            "MODE2/2336" => Ok(TrackType::Mode(1, 2336)),
            "MODE2/2352" => Ok(TrackType::Mode(1, 2352)),
            "CDI/2336" => Ok(TrackType::Cdi(2336)),
            "CDI/2352" => Ok(TrackType::Cdi(2352)),
            _ => Err(format!("Unknown track type: {:?}", s).into()),
        }
    }
}

/// Parse CUE sheet provided by the parameter `source`.
pub fn parse_cue(source: &str) -> Result<Vec<Command>, Error> {
    let mut tokens = tokenize(source)?;
    let mut commands = Vec::new();

    while tokens.len() > 0 {
        commands.push(Command::consume(&mut tokens)?);
    }

    Ok(commands)
}
