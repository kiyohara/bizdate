//! unit test で共有する祝日 fixture、固定時刻、一時ディレクトリ。

use std::{
    env, fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use jiff::Timestamp;

pub(crate) const FIXTURE: &str = include_str!("../tests/fixtures/holidays.csv");

pub(crate) fn now() -> Timestamp {
    "2026-09-06T02:24:33Z".parse().unwrap()
}

pub(crate) struct TempDir(pub(crate) PathBuf);

impl TempDir {
    pub(crate) fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = env::temp_dir().join(format!(
            "bizdate-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // unwind 中も後始末を試み、削除失敗で元のテストエラーを覆い隠さない。
        let _ = fs::remove_dir_all(&self.0);
    }
}
