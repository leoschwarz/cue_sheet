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

//! Provides cue sheet parsing.
//!
//! The [Hydrogenaudio Knowledgebase](http://wiki.hydrogenaud.io/index.php?title=Cue_sheet) might
//! have some more information for you.
//!
//! Additionally [GNU ccd2cue](https://www.gnu.org/software/ccd2cue/) has some more relevant docs.

#![deny(missing_docs)]

#[macro_use]
extern crate error_chain;

pub mod errors;
pub mod parser;
pub mod tracklist;
