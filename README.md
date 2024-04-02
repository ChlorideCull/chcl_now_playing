# chcl_now_playing
A plugin for [LCDSmartie](https://github.com/LCD-Smartie/LCDSmartie) that brings over "now playing" information from the Windows 10/11 MediaControl API, used by Spotify among others.

Initially released on [World Autism Awareness Day 2024](https://www.un.org/en/observances/autism-day) because what is more appropriate for the day than writing Rust for the first time to interface with a poorly documented C API, and an async WinRT API?

# Building
You know, usual cargo deal. `cargo build -r`, then throw `chcl_now_playing.dll` in the plugins directory of LCDSmartie.
