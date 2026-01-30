use {
    crate::{
        clock::fixed_clock::FixedClock, config::Config, file_handler::in_mem::InMemFileHandler,
        log_redirect_generic,
    },
    chrono::Utc,
    std::{
        io::Cursor,
        path::{Path, PathBuf},
        time::Duration,
    },
};

const LOGS: &str = r#"Line1.
Line2.
Line3: with more text in it.
Line4: { "with_json": true }
Line5.
Line6.
Line7.
Line8.
Line9.
"#;

#[tokio::test]
async fn test_log_ration() {
    let config = Config {
        dir: Path::new(".").to_path_buf(),
        prefix: "testing.log".into(),
    };

    let input = Cursor::new(LOGS.as_bytes());
    let clock = FixedClock::new(vec![
        Duration::from_nanos(3),
        Duration::from_secs(1),
        Duration::from_hours(24),
        Duration::from_millis(2),
    ]);
    let mut file_handler = InMemFileHandler::default();

    log_redirect_generic(input, &config, clock, &mut file_handler)
        .await
        .unwrap();

    let mut files: Vec<(PathBuf, Vec<u8>)> = file_handler.into_inner().into_iter().collect();
    files.sort_unstable();

    let mut date = Utc::now().date_naive();
    let mut expected = Vec::new();

    // The first two lines are in the first file because the first date change happens
    // in the third time increment.
    expected.push((
        format!("./{}-{}", config.prefix, date.format("%Y%m%d")),
        &LOGS[0..14],
    ));

    // The next two lines are in the next file.
    date = date.succ_opt().unwrap();
    expected.push((
        format!("./{}-{}", config.prefix, date.format("%Y%m%d")),
        &LOGS[14..72],
    ));

    // Each line is it its own file after that because the default increment is 24 hours.
    for i in 0..5 {
        date = date.succ_opt().unwrap();
        let start = 72 + i * 7;
        let end = 72 + (i + 1) * 7;
        expected.push((
            format!("./{}-{}", config.prefix, date.format("%Y%m%d")),
            &LOGS[start..end],
        ));
    }

    for ((path, contents), (expected_path, expected_contents)) in files.into_iter().zip(expected) {
        let contents = String::from_utf8(contents).unwrap();
        assert_eq!(path, expected_path);
        assert_eq!(contents, expected_contents);
    }
}
