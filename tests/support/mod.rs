use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

pub const BINARY: &str = env!("CARGO_BIN_EXE_bizdate");

pub struct Sandbox(pub PathBuf);

impl Sandbox {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "bizdate-e2e-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    pub fn data_path(&self) -> PathBuf {
        self.0.join("data/bizdate/holidays/holidays.csv")
    }

    pub fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        // 親の設定や proxy に依存せず、実データへの fallback も隔離する。
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", &self.0)
            .env("XDG_DATA_HOME", self.0.join("data"))
            .env("TZ", "UTC")
            .env("BIZDATE_TZ", "UTC")
            .current_dir(&self.0);
        command
    }

    pub fn run(&self, args: &[&str]) -> Output {
        self.command(BINARY).args(args).output().unwrap()
    }

    pub fn write_data(&self, fetched_at: &str) {
        let path = self.data_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!(
            "# bizdate-meta schema=1\n# fetched_at={fetched_at}\n# source_url=https://example.com/holidays.csv\n{}",
            csv()
        )).unwrap();
    }

    pub fn seed(&self) {
        self.write_data(&jiff::Timestamp::now().to_string());
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn csv() -> String {
    // 既存 fixture に月の端を除外する架空の休日を追加する。実際の祝日一覧ではない。
    let body = include_str!("../fixtures/holidays.csv")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{body}\n2026/5/1,テスト休日\n2026/8/31,テスト休日\n")
}

pub fn assert_output(output: Output, code: i32, stdout: &str) {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert_eq!(output.stdout, stdout.as_bytes(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

pub fn assert_error(output: Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains(diagnostic),
        "{stderr:?} lacks {diagnostic:?}"
    );
}

// 1 回だけ GET に応答する。bind を先に済ませ、起動待ちの sleep を不要にする。
// 接続・読み取りには上限を置き、CLI の配線が壊れても server が待ち続けないようにする。
pub fn serve(status: &str, body: Vec<u8>) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/holidays.csv", listener.local_addr().unwrap());
    let status = status.to_owned();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "CLI did not request the fixture");
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line, "GET /holidays.csv HTTP/1.1\r\n");
        loop {
            line.clear();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            if line == "\r\n" {
                break;
            }
        }
        // charset 無しの Shift_JIS も HTTP 層から保存まで通す。
        write!(stream, "HTTP/1.1 {status}\r\nContent-Type: text/csv\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
        stream.write_all(&body).unwrap();
    });
    (url, handle)
}
