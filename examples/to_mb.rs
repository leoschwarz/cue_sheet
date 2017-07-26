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

//! Convert a cue sheet into a tracklist that can be parsed by the MusicBrainz tracklist parser for
//! easy importing of metadata.
//!
//! Note there is one caveat that by only dealing with the data from the cuefile and not the actual
//! source files, this currently results in the last track of the list having an unknown length.
//! This could be fixed (TODO) in the future by providing an option in the Tracklist parser, to
//! also query the specified file lengths, but of course this won't always be applicable.

extern crate cue_sheet;

use cue_sheet::tracklist::Tracklist;
use cue_sheet::errors::Error;

use std::env;
use std::io::Read;
use std::fs::File;

fn perform_conversion(source: &str) -> Result<(), Error> {
    let mut tracklist = Tracklist::parse(source)?;
    // TODO support multi-cds
    assert_eq!(tracklist.files.len(), 1);

    let file = tracklist.files.remove(0);
    for ref t in file.tracks {
        let duration = match t.duration.clone() {
            Some(time) => time.to_string_2(),
            None => "??:??".to_string(),
        };
        println!(
            "{:02} {} - {} {}",
            t.number,
            t.title,
            t.performer.clone().ok_or_else(|| {
                Error::from("Not all tracks have a specified performer.")
            })?,
            duration
        );
    }

    Ok(())
}

fn main() {
    if let Some(path) = env::args().nth(1) {
        // Try reading the file provided by the path.
        let mut file = File::open(path).expect("Failed reading file.");
        let mut content = String::new();
        file.read_to_string(&mut content);

        perform_conversion(content.as_str()).expect("Conversion failed.");
    } else {
        println!(
            "provide a path to a .cue file to be converted into a MusicBrainz compatible tracklist."
        )
    }
}
