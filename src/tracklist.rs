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

//! Generate a tracklist from a cue file.

// TODO don't swallow errors in parsing but use Result and Option where appropriate.

use errors::Error;
use parser::{self, Command, FileFormat, Time, TrackType};

/// A tracklist provides a more useful representation of the information of a cue sheet.
#[derive(Clone, Debug)]
pub struct Tracklist {
    /// Files described by the tracklist.
    pub files: Vec<TrackFile>,

    /// Performer of the tracklist.
    pub performer: Option<String>,

    /// Title of the tracklist.
    pub title: String,
}

impl Tracklist {
    pub fn parse(source: &str) -> Result<Tracklist, Error> {
        let mut commands = parser::parse_cue(source)?;

        let mut performer = None;
        let mut title = None;

        while commands.len() > 0 {
            match commands[0].clone() {
                Command::Performer(p) => {
                    performer = Some(p);
                    commands.remove(0);
                }
                Command::Title(t) => {
                    title = Some(t);
                    commands.remove(0);
                }
                Command::Rem(_, _) => {
                    commands.remove(0);
                }
                _ => {
                    break;
                }
            }
        }

        let mut files = Vec::new();
        while commands.len() > 0 {
            if let Ok(file) = TrackFile::consume(&mut commands) {
                files.push(file);
            } else {
                break;
            }
        }

        if title.is_none() {
            Err("Tracklist can't have no title.".into())
        } else {
            Ok(Tracklist {
                files: files,
                performer: performer,
                title: title.unwrap(),
            })
        }
    }
}

/// One file described by a tracklist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackFile {
    /// List of tracks contained in the file.
    pub tracks: Vec<Track>,

    /// The filename.
    pub name: String,

    /// The format of the file.
    pub format: FileFormat,
}

impl TrackFile {
    fn consume(commands: &mut Vec<Command>) -> Result<Self, Error> {
        if let Command::File(name, format) = commands.remove(0) {
            let mut tracks: Vec<Track> = Vec::new();
            let mut last_time: Option<Time> = None;

            while commands.len() > 0 {
                if let Ok((track, indexes)) = Track::consume(commands) {
                    if indexes.len() > 0 {
                        let time = indexes[indexes.len() - 1].clone();

                        if let Some(start) = last_time {
                            let stop = indexes[0].clone().1;
                            let duration = stop - start;

                            let track_n = tracks.len();
                            if let Some(last_track) = tracks.get_mut(track_n - 1) {
                                (*last_track).duration = Some(duration);
                            }
                        }

                        last_time = Some(time.1);
                    } else {
                        last_time = None;
                    }

                    tracks.push(track);
                } else {
                    break;
                }
            }
            Ok(TrackFile {
                tracks: tracks,
                name: name,
                format: format,
            })
        } else {
            Err(
                "TrackFile::consume called but no Track command found.".into(),
            )
        }
    }
}

/// One track described by a tracklist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Track {
    pub title: String,
    pub track_type: TrackType,
    pub duration: Option<Time>,
    pub number: u32,
    pub performer: Option<String>,
}

type Index = (u32, Time);

impl Track {
    fn consume(commands: &mut Vec<Command>) -> Result<(Track, Vec<Index>), Error> {
        if let Command::Track(track_num, track_type) = commands.remove(0) {
            let mut title = None;
            let mut performer = None;
            let mut index = Vec::new();

            while commands.len() > 0 {
                match commands[0].clone() {
                    Command::Performer(p) => {
                        performer = Some(p);
                        commands.remove(0);
                    }
                    Command::Title(t) => {
                        title = Some(t);
                        commands.remove(0);
                    }
                    Command::Index(i, time) => {
                        index.push((i, time));
                        commands.remove(0);
                    }
                    _ => break,
                }
            }

            if title.is_none() {
                Err("Track can't have no title.".into())
            } else {
                Ok((
                    Track {
                        title: title.unwrap(),
                        track_type: track_type,
                        duration: None,
                        number: track_num,
                        performer: performer,
                    },
                    index,
                ))
            }
        } else {
            Err("Track::consume called but no Track command found.".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() {
        let source = r#"REM GENRE Alternative
                        REM DATE 1991
                        REM DISCID 860B640B
                        REM COMMENT "ExactAudioCopy v0.95b4"
                        PERFORMER "My Bloody Valentine"
                        TITLE "Loveless"
                        FILE "My Bloody Valentine - Loveless.wav" WAVE
                          TRACK 01 AUDIO
                            TITLE "Only Shallow"
                            PERFORMER "My Bloody Valentine"
                            INDEX 01 00:00:00
                          TRACK 02 AUDIO
                            TITLE "Loomer"
                            PERFORMER "My Bloody Valentine"
                            INDEX 01 04:17:52"#;

        let tracklist = Tracklist::parse(source).unwrap();
        assert_eq!(tracklist.title, "Loveless".to_string());

        let files = tracklist.files;
        assert_eq!(files.len(), 1);

        let ref f = files[0];
        assert_eq!(f.name, "My Bloody Valentine - Loveless.wav".to_string());
        assert_eq!(f.format, FileFormat::Wave);

        let ref tracks = f.tracks;
        assert_eq!(tracks.len(), 2);

        assert_eq!(tracks[0].title, "Only Shallow".to_string());
        assert_eq!(tracks[0].track_type, TrackType::Audio);
        assert_eq!(tracks[0].duration, Some(Time::new(4, 17, 52)));
        assert_eq!(tracks[0].number, 1);
        assert_eq!(tracks[0].performer, Some("My Bloody Valentine".to_string()));

        assert_eq!(tracks[1].title, "Loomer".to_string());
        assert_eq!(tracks[1].track_type, TrackType::Audio);
        assert_eq!(tracks[1].duration, None);
        assert_eq!(tracks[1].number, 2);
        assert_eq!(tracks[1].performer, Some("My Bloody Valentine".to_string()));
    }
}