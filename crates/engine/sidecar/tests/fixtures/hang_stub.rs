// Task 021 fixture: a hanging ffmpeg/ffprobe stub for the process-tree
// reaping test (`process_tree_reaping_kills_grandchild_on_probe_timeout` in
// `../process_tree_reaping.rs`).
//
// Not part of the arama workspace and never invoked by any production code
// path - the native-smoke workflow compiles this directly with `rustc`
// (build instructions in that test's own module docs) and places the result
// at the `ffmpeg`/`ffmpeg.exe` and `ffprobe`/`ffprobe.exe` names inside a
// scratch directory used as a Selected-directory discovery target.
//
// Invoked plainly (as arama's discovery invokes "ffmpeg -version"): spawns a
// grandchild of itself, records nothing, then hangs forever without ever
// answering - this is deliberate. The grandchild is the thing under test:
// it must not survive process-tree reaping when arama's probe times out and
// kills the direct child.
//
// Invoked with `--grandchild` (how it re-invokes itself): writes its own PID
// to `grandchild.pid` next to this executable, then also hangs forever. The
// marker file is what the test polls to confirm the grandchild existed
// before asserting it is gone after.
use std::{env, fs, process::Command, thread, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe = env::current_exe().expect("current_exe");
    let marker = exe.with_file_name("grandchild.pid");

    if args.get(1).map(String::as_str) == Some("--grandchild") {
        fs::write(&marker, std::process::id().to_string()).expect("write grandchild marker");
        hang();
    }

    Command::new(&exe)
        .arg("--grandchild")
        .spawn()
        .expect("spawn grandchild");
    hang();
}

fn hang() -> ! {
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}
