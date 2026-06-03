use assert_cmd::Command;
use std::io::Write;

fn cmd() -> Command {
    Command::cargo_bin("zztop").unwrap()
}

#[test]
fn empty_datafile_exits_nonzero_with_empty_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("z");
    std::fs::File::create(&p).unwrap();

    let assert = cmd()
        .env("_Z_DATA", &p)
        .assert()
        .failure();

    let output = assert.get_output();
    assert!(output.stdout.is_empty(), "expected empty stdout, got {:?}", output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no zsh-z data"), "stderr was {stderr:?}");
}

#[test]
fn missing_datafile_exits_nonzero_with_empty_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("does-not-exist");

    let assert = cmd()
        .env("_Z_DATA", &p)
        .assert()
        .failure();

    let output = assert.get_output();
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no zsh-z data"), "stderr was {stderr:?}");
}

#[test]
fn all_paths_missing_on_disk_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("z");
    let mut f = std::fs::File::create(&p).unwrap();
    writeln!(f, "/definitely/does/not/exist/aaa|10.0|1700000000").unwrap();
    writeln!(f, "/definitely/does/not/exist/bbb|5.0|1700000000").unwrap();

    let assert = cmd()
        .env("_Z_DATA", &p)
        .assert()
        .failure();

    let output = assert.get_output();
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no existing paths"),
        "stderr was {stderr:?}"
    );
}

#[test]
fn no_tty_exits_nonzero_without_emitting_path() {
    // When the binary has valid data but no TTY (stdin/stdout piped under
    // assert_cmd), dialoguer should fail to interact and we should exit
    // non-zero with no path on stdout.
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real");
    std::fs::create_dir(&real).unwrap();

    let datafile = dir.path().join("z");
    let mut f = std::fs::File::create(&datafile).unwrap();
    writeln!(f, "{}|1.0|1700000000", real.display()).unwrap();

    let output = cmd()
        .env("_Z_DATA", &datafile)
        .output()
        .unwrap();

    assert!(!output.status.success(), "expected non-zero exit");
    assert!(
        output.stdout.is_empty(),
        "expected empty stdout, got {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
}
