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
use std::cmp::Ordering;
use std::fmt;
use std::ops::Sub;
use std::str::FromStr;

mod tokenization;
use self::tokenization::tokenize;
pub use self::tokenization::Token;

mod command;
pub use self::command::Command;

/// Number of audio frames/sectors per second in cue sheets.
///
/// This value is supposed to be fixed for all cue sheets to 75 frames per second.
/// TODO: Double-check, how does this interact with the media type?
const FPS: i64 = 75;

/// Time representation of the format `mm:ss:ff`.
///
/// Where mm = minutes, ss = seconds, ff = frames/sectors.
/// There are 75 frames per second, 60 seconds per minute.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Time {
    /// Minutes time component.
    mins: i32,

    /// Seconds time component.
    secs: i8,

    /// Frames time component.
    frames: i8,
}

impl Time {
    /// Create a new instance with the specified components.
    pub fn new(minutes: i32, seconds: i8, frames: i8) -> Time {
        Time {
            mins: minutes,
            secs: seconds,
            frames: frames,
        }
    }

    /// Format as `mm:ss' dropping truncating the remainding frames.
    pub fn to_string_2(&self) -> String {
        format!("{:02}:{:02}", self.mins, self.secs)
    }

    /// Returns the "minutes" component of this instance.
    ///
    /// ```
    /// use cue_sheet::parser::Time;
    ///
    /// let time = Time::new(1, 2, 3);
    /// assert_eq!(time.minutes(), 1);
    /// ```
    pub fn minutes(&self) -> i32 {
        self.mins
    }

    /// Returns the "seconds" component of this instance.
    ///
    /// ```
    /// use cue_sheet::parser::Time;
    ///
    /// let time = Time::new(1, 2, 3);
    /// assert_eq!(time.seconds(), 2);
    /// ```
    pub fn seconds(&self) -> i8 {
        self.secs
    }

    /// Returns the "frames/sectors" component of this instance.
    ///
    /// ```
    /// use cue_sheet::parser::Time;
    ///
    /// let time = Time::new(1, 2, 3);
    /// assert_eq!(time.frames(), 3);
    /// ```
    pub fn frames(&self) -> i8 {
        self.frames
    }

    /// Returns the total number of minutes represented by this instance.
    ///
    /// ```
    /// use cue_sheet::parser::Time;
    ///
    /// let time = Time::new(1, 2, 3);
    /// assert_eq!(time.total_minutes(), 1.034);
    ///
    /// let time = Time::new(2, 30, 0);
    /// assert_eq!(time.total_minutes(), 2.5);
    /// ```
    pub fn total_minutes(&self) -> f64 {
        self.total_seconds() / 60.
    }

    /// Returns the total number of seconds represented by this instance.
    ///
    /// ```
    /// use cue_sheet::parser::Time;
    ///
    /// let time = Time::new(1, 2, 3);
    /// assert_eq!(time.total_seconds(), 62.04);
    ///
    /// let time = Time::new(2, 30, 0);
    /// assert_eq!(time.total_seconds(), 150.);
    /// ```
    pub fn total_seconds(&self) -> f64 {
        self.mins as f64 * 60. + self.secs as f64 + (self.frames as f64) / (FPS as f64)
    }

    /// Returns the total number of frames/sectors represented by this instance.
    ///
    /// ```
    /// use cue_sheet::parser::Time;
    ///
    /// let time = Time::new(1, 2, 3);
    /// assert_eq!(time.total_frames(), 4653);
    ///
    /// let time = Time::new(2, 30, 5);
    /// assert_eq!(time.total_frames(), 11255);
    /// ```
    pub fn total_frames(&self) -> i64 {
        (self.mins as i64 * 60 + self.secs as i64) * FPS + self.frames as i64
    }

    /// Create an instance for the specified number of frames/sectors.
    ///
    /// ```
    /// use cue_sheet::parser::Time;
    ///
    /// let time = Time::from_frames(200);
    /// assert_eq!(time, Time::new(0, 2, 50));
    /// ```
    pub fn from_frames(from: i64) -> Time {
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

impl Ord for Time {
    fn cmp(&self, other: &Time) -> Ordering {
        self.total_frames().cmp(&other.total_frames())
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Time) -> Option<Ordering> {
        Some(self.cmp(other))
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
        let left = self.total_frames();
        let right = rhs.total_frames();

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
        match s.to_uppercase().as_str() {
            "WAVE" => Ok(FileFormat::Wave),
            "MP3" => Ok(FileFormat::Mp3),
            "AIFF" => Ok(FileFormat::Aiff),
            "BINARY" => Ok(FileFormat::Binary),
            "MOTOROLA" => Ok(FileFormat::Motorola),
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
        match s.to_uppercase().as_str() {
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
        match s.to_uppercase().as_str() {
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
