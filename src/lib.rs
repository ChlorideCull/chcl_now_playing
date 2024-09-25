// Copyright 2024 Sebastian "Dusty the Fuzzy Dragon" Johansson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
mod plugin;
use std::ffi::c_char;

use lcdsmartie_rs::ShortString;

const DOC_LINE: &str = 
"# Get the currently playing artist
$dll(chcl_now_playing,1,,)
# Get the currently playing song title
$dll(chcl_now_playing,2,,)
# Get the current song progress - MM:SS
$dll(chcl_now_playing,3,,) / $dll(chcl_now_playing,4,,)
# Get the current song progress as a bar 20 characters wide
$Bar($dll(chcl_now_playing,5,,),$dll(chcl_now_playing,6,,),20)
";

struct NowPlaying {}

impl lcdsmartie_rs::Plugin for NowPlaying {
    fn new() -> Self {
        NowPlaying {}
    }

    fn developer(&self) -> &'static str {
        "Dusty the Fuzzy Dragon"
    }
    
    fn version(&self) -> &'static str {
        "1.RIIR"
    }

    fn documentation(&self) -> lcdsmartie_rs::ShortString {
        DOC_LINE.try_into().unwrap()
    }

    fn minimum_refresh_interval_ms(&self) -> i32 {
        150
    }

    fn function_router(&self, fid: u8, _: &str, _: &str) -> Result<ShortString, String> {
        plugin::ensure_initialized();
        let guard = match fid {
            1 => plugin::ARTIST.lock_blocking(),
            2 => plugin::TITLE.lock_blocking(),
            3 => plugin::POSITION.lock_blocking(),
            4 => plugin::LENGTH.lock_blocking(),
            5 => plugin::POSITION_I.lock_blocking(),
            6 => plugin::LENGTH_I.lock_blocking(),
            7 => plugin::STATUS.lock_blocking(),
            _ => unimplemented!()
        };
        if let Some(v) = guard.as_ref() {
            let res: Result<ShortString, _> = v.as_str().try_into();
            if res.is_err() {
                return Err(res.err().unwrap());
            }
        }
        return match fid {
            5..=6 => Ok("0".try_into().unwrap()),
            _ => Ok(lcdsmartie_rs::ShortString::default())
        };
    }
}

impl Drop for NowPlaying {
    fn drop(&mut self) {
        plugin::cleanup_state();
    }
}

lcdsmartie_rs::define_plugin!(NowPlaying);