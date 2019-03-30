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
    pub title: Option<String>,
}

impl Tracklist {
    /// Parse a cue sheet (content provided as `source`) into a `Tracklist`.
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

        Ok(Tracklist {
            files: files,
            performer: performer,
            title: title,
        })
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
                if let Ok(track) = Track::consume(commands) {
                    if track.index.len() > 0 {
                        let time = track.index[track.index.len() - 1].clone();

                        if let Some(start) = last_time {
                            let stop = track.index[0].clone().1;
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
            Err("TrackFile::consume called but no Track command found.".into())
        }
    }
}

/// One track described by a tracklist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Track {
    /// Title of the track.
    pub title: Option<String>,

    /// Type of the track.
    pub track_type: TrackType,

    /// Duration of the track, if it was possible to determine it.
    ///
    /// This is only possible if tracks have index commands attached to them.
    /// Also note that with just a cue file it is usually not possible to determine the duration of
    /// the last track in the list.
    pub duration: Option<Time>,

    /// Index commands attached to this track (if any).
    pub index: Vec<Index>,

    /// Track number as provided in the cue sheet.
    pub number: u32,

    /// The performer of the track if any was stated.
    pub performer: Option<String>,
}

type Index = (u32, Time);

impl Track {
    fn consume(commands: &mut Vec<Command>) -> Result<Track, Error> {
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
                    Command::Pregap(time) => {
                        let next_command = commands
                            .get(1)
                            .ok_or("Pregap is the last command in the track!".to_owned())?
                            .to_owned();

                        let first_index;
                        match next_command {
                            Command::Index(_, time) => first_index = time,
                            _ => {
                                return Err("Pregap is not followed by an index!".into());
                            }
                        }
                        let diff = first_index.total_frames() - time.total_frames();
                        index.push((0, Time::from_frames(diff)));
                        commands.remove(0);
                    }
                    Command::Index(i, time) => {
                        index.push((i, time));
                        commands.remove(0);
                    }
                    _ => break,
                }
            }

            Ok(Track {
                title: title,
                track_type: track_type,
                duration: None,
                index: index,
                number: track_num,
                performer: performer,
            })
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
        assert_eq!(tracklist.title.unwrap(), "Loveless".to_string());

        let files = tracklist.files;
        assert_eq!(files.len(), 1);

        let ref f = files[0];
        assert_eq!(f.name, "My Bloody Valentine - Loveless.wav".to_string());
        assert_eq!(f.format, FileFormat::Wave);

        let ref tracks = f.tracks;
        assert_eq!(tracks.len(), 2);

        assert_eq!(tracks[0].clone().title.unwrap(), "Only Shallow".to_string());
        assert_eq!(tracks[0].track_type, TrackType::Audio);
        assert_eq!(tracks[0].duration, Some(Time::new(4, 17, 52)));
        assert_eq!(tracks[0].number, 1);
        assert_eq!(tracks[0].performer, Some("My Bloody Valentine".to_string()));

        assert_eq!(tracks[1].clone().title.unwrap(), "Loomer".to_string());
        assert_eq!(tracks[1].track_type, TrackType::Audio);
        assert_eq!(tracks[1].duration, None);
        assert_eq!(tracks[1].number, 2);
        assert_eq!(tracks[1].performer, Some("My Bloody Valentine".to_string()));
    }

    #[test]
    fn pregap() {
        let src = r#"FILE "disc.img" BINARY
                       TRACK 01 MODE1/2352
                         INDEX 01 00:00:00
                       TRACK 02 AUDIO
                         PREGAP 00:02:00
                         INDEX 01 58:41:36
                       TRACK 03 AUDIO
                         INDEX 00 61:06:08
                         INDEX 01 61:08:08"#;

        let tracklist = Tracklist::parse(src).unwrap();

        let ref f = tracklist.files[0];
        let ref tracks = f.tracks;

        assert_eq!(tracks[0].index[0], (1, Time::new(0, 0, 0)));
        assert_eq!(tracks[1].index[0], (0, Time::new(58, 39, 36)));
        assert_eq!(tracks[1].index[1], (1, Time::new(58, 41, 36)));
        assert_eq!(tracks[2].index[0], (0, Time::new(61, 06, 08)));
        assert_eq!(tracks[2].index[1], (1, Time::new(61, 08, 08)));
    }
}
