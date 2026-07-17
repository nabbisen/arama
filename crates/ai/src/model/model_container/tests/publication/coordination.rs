use super::*;

use std::{path::PathBuf, process::Command};

#[test]
fn model_lock_subprocess_helper() {
    let Some(root) = std::env::var_os("ARAMA_MODEL_LOCK_TEST_ROOT") else {
        return;
    };
    let name = std::env::var("ARAMA_MODEL_LOCK_TEST_NAME").expect("model lock test name");
    let acquired = PathBuf::from(
        std::env::var_os("ARAMA_MODEL_LOCK_TEST_ACQUIRED").expect("acquired marker path"),
    );
    let released = PathBuf::from(
        std::env::var_os("ARAMA_MODEL_LOCK_TEST_RELEASED").expect("released marker path"),
    );
    let hold_ms = std::env::var("ARAMA_MODEL_LOCK_TEST_HOLD_MS")
        .expect("hold milliseconds")
        .parse::<u64>()
        .expect("numeric hold milliseconds");

    test_runtime().block_on(async {
        let _lock = acquire_model_lock(std::path::Path::new(&root), &name)
            .await
            .expect("acquire subprocess model lock");
        fs::write(&acquired, operation_clock().to_string()).expect("write acquired marker");
        thread::sleep(Duration::from_millis(hold_ms));
        fs::write(&released, operation_clock().to_string()).expect("write released marker");
    });
}

fn operation_clock() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after UNIX epoch")
        .as_nanos()
}

#[test]
fn independent_processes_serialize_one_model_directory() {
    let root = tempfile::tempdir().expect("temporary model root");
    let first_acquired = root.path().join("first-acquired");
    let first_released = root.path().join("first-released");
    let second_acquired = root.path().join("second-acquired");
    let second_released = root.path().join("second-released");
    let test_name =
        "model::model_container::tests::publication::coordination::model_lock_subprocess_helper";
    let executable = std::env::current_exe().expect("current test executable");
    let spawn = |acquired: &std::path::Path,
                 released: &std::path::Path,
                 hold_ms: &str|
     -> std::process::Child {
        Command::new(&executable)
            .arg("--exact")
            .arg(test_name)
            .arg("--nocapture")
            .env("ARAMA_MODEL_LOCK_TEST_ROOT", root.path())
            .env("ARAMA_MODEL_LOCK_TEST_NAME", "shared-model")
            .env("ARAMA_MODEL_LOCK_TEST_ACQUIRED", acquired)
            .env("ARAMA_MODEL_LOCK_TEST_RELEASED", released)
            .env("ARAMA_MODEL_LOCK_TEST_HOLD_MS", hold_ms)
            .spawn()
            .expect("spawn model lock helper")
    };

    let mut first = spawn(&first_acquired, &first_released, "300");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !first_acquired.exists() && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    assert!(
        first_acquired.exists(),
        "first process did not acquire lock"
    );
    let mut second = spawn(&second_acquired, &second_released, "0");

    assert!(first.wait().expect("wait for first helper").success());
    assert!(second.wait().expect("wait for second helper").success());
    let first_release = fs::read_to_string(first_released)
        .expect("first release marker")
        .parse::<u128>()
        .expect("first release time");
    let second_acquire = fs::read_to_string(second_acquired)
        .expect("second acquire marker")
        .parse::<u128>()
        .expect("second acquire time");
    assert!(second_acquire >= first_release);
}

#[test]
fn model_lock_wait_is_bounded_and_reports_cross_process_contention() {
    let root = tempfile::tempdir().expect("temporary model root");
    test_runtime().block_on(async {
        let first = acquire_model_lock(root.path(), "contended-model")
            .await
            .expect("acquire first lock");
        let error = acquire_model_lock_with_timeout(
            root.path(),
            "contended-model",
            Duration::from_millis(40),
        )
        .await
        .expect_err("second lock must time out");
        assert!(
            error
                .to_string()
                .contains("waiting for another application instance")
        );
        drop(first);
    });
}

#[test]
fn persisted_operation_sequence_is_monotonic_without_wall_clock_ordering() {
    let root = tempfile::tempdir().expect("temporary model root");
    let sequence_path = root.path().join(".ordered-model.sequence");
    fs::write(&sequence_path, "41").expect("seed persisted sequence");

    let (first, second) = test_runtime().block_on(async {
        let first = next_operation_sequence(root.path(), "ordered-model")
            .await
            .expect("first sequence");
        let second = next_operation_sequence(root.path(), "ordered-model")
            .await
            .expect("second sequence");
        (first, second)
    });

    assert_eq!((first, second), (42, 43));
    assert_eq!(
        fs::read_to_string(sequence_path).expect("stored sequence"),
        "43"
    );
}
