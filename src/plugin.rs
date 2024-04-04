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
use std::{ffi::CString, thread::{self, JoinHandle}, time::Duration};
use chrono::{TimeZone, Utc};
use smol::{lock::Mutex, Timer};
use windows::{core::{Error, HSTRING}, Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus};

pub struct PluginState {
    pub(crate) error: Option<Error>,
    pub(crate) error_src: String,
    pub(crate) stop: bool
}

pub static TITLE: smol::lock::Mutex<Option<CString>> = Mutex::new(None);
pub static ARTIST: smol::lock::Mutex<Option<CString>> = Mutex::new(None);
pub static POSITION: smol::lock::Mutex<Option<CString>> = Mutex::new(None);
pub static POSITION_I: smol::lock::Mutex<Option<CString>> = Mutex::new(None);
pub static LENGTH: smol::lock::Mutex<Option<CString>> = Mutex::new(None);
pub static LENGTH_I: smol::lock::Mutex<Option<CString>> = Mutex::new(None);
pub static STATUS: smol::lock::Mutex<Option<CString>> = Mutex::new(None);

pub static mut STATE: Option<smol::lock::Mutex<PluginState>> = Option::None;
pub static mut THREADHANDLE: Option<JoinHandle<()>> = Option::None;

pub fn init_state() {
    unsafe {
        STATE = Some(smol::lock::Mutex::new(PluginState {
            error: None,
            error_src: String::new(),
            stop: false
        }));
    }
}

pub fn start_refresh_thread() -> JoinHandle<()> {
    return thread::spawn(refresh_thread);
}

pub fn ensure_initialized() {
    if unsafe { STATE.as_ref() }.is_none() {
        init_state();
        unsafe { THREADHANDLE = Some(start_refresh_thread()); }
    }
}

pub fn cleanup_state() {
    if unsafe { STATE.as_ref() }.is_some() {
        let mut guard = unsafe { STATE.as_ref() }.unwrap().lock_blocking();
        guard.stop = true;
        drop(guard);
        unsafe {
            THREADHANDLE.take().expect("There should be a thread to clean up when STATE is set").join().unwrap();
        }
    }
}

fn refresh_thread() {
    let state = unsafe { STATE.as_ref().expect("State not initialized") };
    smol::block_on(wrt_refresh_thread(state));
}

async fn protected_set<T>(mutex: &smol::lock::Mutex<T>, val: T) {
    let mut guard = mutex.lock().await;
    *guard = val;
    drop(guard);
}

fn format_playback_status(status: &GlobalSystemMediaTransportControlsSessionPlaybackStatus) -> &str {
    match *status {
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => "Changing",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => "Closed",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => "Opened",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => "Paused",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => "Playing",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => "Stopped",
        _ => "Unknown"
    }
}

pub async fn wrt_refresh_thread(state: &smol::lock::Mutex<PluginState>) {
    let manager = windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .expect("Failed to get GlobalSystemMediaTransportControlsSessionManager promise")
        .await.expect("Failed to get GlobalSystemMediaTransportControlsSessionManager");
    loop {
        let guard = state.lock().await;
        if guard.stop {
            break;
        }
        drop(guard);

        Timer::after(Duration::from_millis(100)).await;
        let session = manager.GetCurrentSession();
        if session.is_err() {
            protected_set(&TITLE, None).await;
            protected_set(&ARTIST, None).await;
            protected_set(&POSITION, None).await;
            protected_set(&LENGTH, None).await;
            continue;
        }
        let session = session.unwrap();
        
        let playback_info = session.GetPlaybackInfo();
        if !state_error_wrap(&playback_info, state, "PI").await {
            continue;
        }
        let playback_info = playback_info.unwrap();
        if let Ok(playback_status) = playback_info.PlaybackStatus() {
            protected_set(&STATUS, Some(CString::new(format_playback_status(&playback_status)).unwrap())).await;
        } else {
            protected_set(&STATUS, Some(CString::new("Stopped").unwrap())).await;
        }

        let timeline_properties = session.GetTimelineProperties();
        if let Ok(timeline_properties) = timeline_properties {
            // The API is updated pretty slowly, so we have to fill in the blanks...
            let offset: Duration;
            if playback_info.PlaybackStatus().ok().unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped) != GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                offset = Duration::ZERO;
            } else {
                let now = chrono::Utc::now();
                let unix_epoch_micros: i64 = 11_644_473_600 * 1_000_000;
                let last_updated_micros = (timeline_properties.LastUpdatedTime().unwrap().UniversalTime / 10) - unix_epoch_micros;
                let last_updated = Utc.timestamp_micros(last_updated_micros).single().unwrap();
                offset = now.signed_duration_since(last_updated).to_std().ok().unwrap_or(Duration::new(0, 0));
            }

            let pos = Duration::from_micros((timeline_properties.Position().unwrap_or_default().Duration / 10).try_into().unwrap()) + offset;
            protected_set(&POSITION, Some(CString::new(format!("{}:{:02}", pos.as_secs() / 60, pos.as_secs() % 60)).unwrap_or(CString::new("CString::New failed on title").unwrap()))).await;
            protected_set(&POSITION_I, Some(CString::new(pos.as_secs().to_string()).unwrap())).await;
            let len = Duration::from_micros((timeline_properties.EndTime().unwrap_or_default().Duration / 10).try_into().unwrap());
            protected_set(&LENGTH, Some(CString::new(format!("{}:{:02}", len.as_secs() / 60, len.as_secs() % 60)).unwrap_or(CString::new("CString::New failed on title").unwrap()))).await;
            protected_set(&LENGTH_I, Some(CString::new(len.as_secs().to_string()).unwrap())).await;
        } else {
            protected_set(&POSITION, None).await;
            protected_set(&POSITION_I, None).await;
            protected_set(&LENGTH, None).await;
            protected_set(&LENGTH_I, None).await;
        }
        
        // TODO: Fails if there is a session, but no media loaded.
        let media_properties = session.TryGetMediaPropertiesAsync()
            .expect("Failed to get media properties promise")
            .await;
        if let Ok(media_properties) = media_properties {
            protected_set(&TITLE, Some(CString::new(media_properties.Title().unwrap_or(HSTRING::new()).to_string_lossy()).unwrap_or(CString::new("CString::New failed on title").unwrap()))).await;
            protected_set(&ARTIST, Some(CString::new(media_properties.Artist().unwrap_or(HSTRING::new()).to_string_lossy()).unwrap_or(CString::new("CString::New failed on title").unwrap()))).await;
        } else {
            protected_set(&TITLE, None).await;
            protected_set(&ARTIST, None).await;
        }
    }
}

async fn state_error_wrap<T>(res: &Result<T, Error>, state: &smol::lock::Mutex<PluginState>, src: &str) -> bool {
    if !res.is_ok() {
        let mut guard = state.lock().await;
        guard.error = Some(res.as_ref().err().unwrap().clone());
        guard.error_src = String::from(src);
        drop(guard);
        return false;
    }
    return true;
}