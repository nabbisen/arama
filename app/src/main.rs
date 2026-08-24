// Task 037: a release build carries no `windows_subsystem` attribute
// today, so Windows treats it as a console app (PE subsystem 3) and opens
// a console window behind the GUI on every launch. `windows_subsystem`
// keeps the debug-build console (still wanted there) and removes it only
// from a release build; the removal's own consequence - `eprintln!` sites
// that relied on that console - is handled by `arama_env::diagnostic`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> iced::Result {
    arama::start()
}
