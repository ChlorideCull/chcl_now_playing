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
use std::ffi::{c_char, c_int, CStr};

#[no_mangle]
pub extern "stdcall" fn SmartieInit() {
    // Deliberately left empty - tried initializing the background thread here, but kept running into an issue where it wasn't loaded.
}

#[no_mangle]
pub extern "stdcall" fn SmartieFini() {
    plugin::cleanup_state();
}

const INFO_LINE: &CStr = c"Developer: Dusty the Fuzzy Dragon\r\nVersion: 1.RIIR";
#[no_mangle]
pub extern "stdcall" fn SmartieInfo() -> *const c_char {
    return INFO_LINE.as_ptr();
}

const DOC_LINE: &CStr = 
c"# Get the currently playing artist
$dll(chcl_now_playing,1,,)
# Get the currently playing song title
$dll(chcl_now_playing,2,,)
# Get the current song progress - MM:SS
$dll(chcl_now_playing,3,,) / $dll(chcl_now_playing,4,,)
# Get the current song progress as a bar 20 characters wide
$Bar($dll(chcl_now_playing,5,,),$dll(chcl_now_playing,6,,),20)
";
#[no_mangle]
pub extern "stdcall" fn SmartieDemo() -> *const c_char {
    return DOC_LINE.as_ptr();
}

#[no_mangle]
pub extern "stdcall" fn GetMinRefreshInterval() -> c_int {
    return 150; // Internal refresh rate is 100 ms + runtime, so 150 is reasonable.
}

#[no_mangle]
pub extern "stdcall" fn function1(_: *const c_char, _: *const c_char) -> *const c_char {
    plugin::ensure_initialized();
    let guard = plugin::ARTIST.lock_blocking();
    if let Some(val) = guard.as_ref() {
        return val.as_ptr();
    } else {
        return c"".as_ptr();
    }
}

#[no_mangle]
pub extern "stdcall" fn function2(_: *const c_char, _: *const c_char) -> *const c_char {
    plugin::ensure_initialized();
    let guard = plugin::TITLE.lock_blocking();
    if let Some(val) = guard.as_ref() {
        return val.as_ptr();
    } else {
        return c"".as_ptr();
    }
}

#[no_mangle]
pub extern "stdcall" fn function3(_: *const c_char, _: *const c_char) -> *const c_char {
    plugin::ensure_initialized();
    let guard = plugin::POSITION.lock_blocking();
    if let Some(val) = guard.as_ref() {
        return val.as_ptr();
    } else {
        return c"".as_ptr();
    }
}

#[no_mangle]
pub extern "stdcall" fn function4(_: *const c_char, _: *const c_char) -> *const c_char {
    plugin::ensure_initialized();
    let guard = plugin::LENGTH.lock_blocking();
    if let Some(val) = guard.as_ref() {
        return val.as_ptr();
    } else {
        return c"".as_ptr();
    }
}

#[no_mangle]
pub extern "stdcall" fn function5(_: *const c_char, _: *const c_char) -> *const c_char {
    plugin::ensure_initialized();
    let guard = plugin::POSITION_I.lock_blocking();
    if let Some(val) = guard.as_ref() {
        return val.as_ptr();
    } else {
        return c"0".as_ptr();
    }
}

#[no_mangle]
pub extern "stdcall" fn function6(_: *const c_char, _: *const c_char) -> *const c_char {
    plugin::ensure_initialized();
    let guard = plugin::LENGTH_I.lock_blocking();
    if let Some(val) = guard.as_ref() {
        return val.as_ptr();
    } else {
        return c"0".as_ptr();
    }
}

#[no_mangle]
pub extern "stdcall" fn function7(_: *const c_char, _: *const c_char) -> *const c_char {
    plugin::ensure_initialized();
    let guard = plugin::STATUS.lock_blocking();
    if let Some(val) = guard.as_ref() {
        return val.as_ptr();
    } else {
        return c"0".as_ptr();
    }
}