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

//! The errors used by this crate.
//!
//! Notice that so far error handling was done rather quickly with a lot of string based error
//! messages.

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links { }

    foreign_links {
        ParseInt(::std::num::ParseIntError)
            #[doc="Parsing a string into an integer failed."];
    }

    errors { }
}
