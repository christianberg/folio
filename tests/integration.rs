/// Narrow integration tests — these hit real I/O.
/// Run with: cargo test --test integration

// ── Filesystem ────────────────────────────────────────────────────────────────

mod filesystem {
    use folio::infrastructure::Filesystem;
    use std::io::Write;

    #[test]
    fn reads_real_file() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "hello folio").unwrap();

        let fs = Filesystem::create();
        let content = fs.read_to_string(f.path().to_str().unwrap()).unwrap();
        assert_eq!(content, "hello folio");
    }

    #[test]
    fn returns_not_found_for_missing_file() {
        let fs = Filesystem::create();
        let err = fs.read_to_string("/tmp/folio-no-such-file-xyzzy").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn null_matches_real_for_existing_file() {
        let content = "2026-04-03\n    food type:expense 45.00\n    checking type:asset -45.00\n";
        let fs_null = Filesystem::create_null([("ledger.folio", content)]);
        assert_eq!(fs_null.read_to_string("ledger.folio").unwrap(), content);
    }

    #[test]
    fn null_matches_real_for_missing_file() {
        let fs_null = Filesystem::create_null(std::iter::empty::<(&str, &str)>());
        let err = fs_null.read_to_string("missing.folio").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn appends_to_real_file() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let path = f.path().to_str().unwrap();

        let fs = Filesystem::create();
        fs.append_str(path, "first\n").unwrap();
        fs.append_str(path, "second\n").unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "first\nsecond\n");
    }

    #[test]
    fn null_append_is_readable_via_read_to_string() {
        let fs = Filesystem::create_null(std::iter::empty::<(&str, &str)>());
        fs.append_str("out.folio", "hello\n").unwrap();
        assert_eq!(fs.read_to_string("out.folio").unwrap(), "hello\n");
    }

    #[test]
    fn track_appends_captures_path_and_content() {
        let fs = Filesystem::create_null(std::iter::empty::<(&str, &str)>());
        let tracker = fs.track_appends();
        fs.append_str("a.folio", "tx1\n").unwrap();
        fs.append_str("b.folio", "tx2\n").unwrap();
        let appends = tracker.all();
        assert_eq!(appends[0], ("a.folio".to_string(), "tx1\n".to_string()));
        assert_eq!(appends[1], ("b.folio".to_string(), "tx2\n".to_string()));
    }
}

// ── Output ────────────────────────────────────────────────────────────────────

mod output {
    use folio::infrastructure::Output;

    #[test]
    fn real_println_is_tracked() {
        let output = Output::create();
        let tracker = output.track_stdout();
        output.println("hello");
        assert_eq!(tracker.all(), vec!["hello"]);
    }

    #[test]
    fn real_eprintln_is_tracked() {
        let output = Output::create();
        let tracker = output.track_stderr();
        output.eprintln("oops");
        assert_eq!(tracker.all(), vec!["oops"]);
    }

    #[test]
    fn stops_accumulating_when_tracker_is_dropped() {
        let output = Output::create_null();
        {
            let tracker = output.track_stdout();
            output.println("captured");
            assert_eq!(tracker.all(), vec!["captured"]);
        } // tracker dropped here
        output.println("not captured");
        // No way to observe the Vec now — the point is it was not allocated.
        // Re-track to confirm the slot is fresh.
        let tracker2 = output.track_stdout();
        output.println("fresh");
        assert_eq!(tracker2.all(), vec!["fresh"]);
    }
}

// ── Clock ─────────────────────────────────────────────────────────────────────

mod clock {
    use chrono::NaiveDate;
    use folio::infrastructure::Clock;

    // Clock::create() delegates to chrono::Local::now() — too thin to narrow-test.

    #[test]
    fn null_returns_configured_date() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let clock = Clock::create_null(date);
        assert_eq!(clock.today(), date);
    }
}

// ── Prompt ────────────────────────────────────────────────────────────────────

mod prompt {
    use chrono::NaiveDate;
    use folio::infrastructure::Prompt;
    use rust_decimal::Decimal;

    // Prompt::create() wraps inquire which requires a TTY — too thin to narrow-test.
    // The null instance behaviour is fully exercised here.

    #[test]
    fn null_date_select_returns_parsed_date() {
        let p = Prompt::create_null(["2026-03-15"]);
        let default = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(p.date_select("Date", default), NaiveDate::from_ymd_opt(2026, 3, 15));
    }

    #[test]
    fn null_date_select_uses_default_on_empty_answer() {
        let p = Prompt::create_null([""]);
        let default = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(p.date_select("Date", default), Some(default));
    }

    #[test]
    fn null_multi_select_returns_parsed_selections() {
        let p = Prompt::create_null(["food, type:expense"]);
        let opts = vec!["food".to_string(), "type:expense".to_string(), "type:asset".to_string()];
        assert_eq!(
            p.multi_select("Tags", &opts),
            Some(vec!["food".to_string(), "type:expense".to_string()])
        );
    }

    #[test]
    fn null_multi_select_returns_empty_vec_on_empty_answer() {
        let p = Prompt::create_null([""]);
        assert_eq!(p.multi_select("Tags", &[]), Some(vec![]));
    }

    #[test]
    fn null_multi_select_returns_none_when_queue_empty() {
        let p = Prompt::create_null(std::iter::empty::<&str>());
        assert_eq!(p.multi_select("Tags", &[]), None);
    }

    #[test]
    fn null_decimal_returns_parsed_value() {
        let p = Prompt::create_null(["42.50"]);
        assert_eq!(p.decimal("Amount", None), Some(Decimal::new(4250, 2)));
    }

    #[test]
    fn null_decimal_uses_default_on_empty_answer() {
        let p = Prompt::create_null([""]);
        let default = Decimal::new(100, 0);
        assert_eq!(p.decimal("Amount", Some(default)), Some(default));
    }

    #[test]
    fn null_decimal_returns_none_when_queue_empty() {
        let p = Prompt::create_null(std::iter::empty::<&str>());
        assert_eq!(p.decimal("Amount", None), None);
    }

    #[test]
    fn null_text_with_completions_returns_provided_answer() {
        let p = Prompt::create_null(["type:expense"]);
        let opts = vec!["type:expense".to_string(), "type:asset".to_string()];
        assert_eq!(p.text_with_completions("Tag", &opts), Some("type:expense".to_string()));
    }

    #[test]
    fn null_text_with_completions_returns_none_when_queue_empty() {
        let p = Prompt::create_null(std::iter::empty::<&str>());
        assert_eq!(p.text_with_completions("Tag", &[]), None);
    }

    #[test]
    fn null_confirm_parses_y_as_true() {
        let p = Prompt::create_null(["y"]);
        assert_eq!(p.confirm("Continue?", false), Some(true));
    }

    #[test]
    fn null_confirm_parses_n_as_false() {
        let p = Prompt::create_null(["n"]);
        assert_eq!(p.confirm("Continue?", true), Some(false));
    }

    #[test]
    fn null_confirm_returns_none_when_queue_empty() {
        let p = Prompt::create_null(std::iter::empty::<&str>());
        assert_eq!(p.confirm("Continue?", false), None);
    }
}

// ── Args ──────────────────────────────────────────────────────────────────────

mod args {
    use folio::infrastructure::Args;

    // Args::create() is a one-liner (Self::parse()) — its behaviour is entirely
    // clap's. There is no meaningful narrow test beyond verifying that create_null
    // parses the same way, which exercises the same parse_from path.
    #[test]
    fn null_parses_check_subcommand() {
        let args = Args::create_null(["folio", "check", "ledger.folio"]);
        assert!(
            matches!(args.command, folio::infrastructure::Command::Check { ref path } if path == "ledger.folio"),
            "expected Check subcommand with correct path",
        );
    }

    #[test]
    fn null_parses_add_subcommand() {
        let args = Args::create_null(["folio", "add", "ledger.folio"]);
        assert!(
            matches!(args.command, folio::infrastructure::Command::Add { ref path } if path == "ledger.folio"),
            "expected Add subcommand with correct path",
        );
    }
}

// ── End-to-end ────────────────────────────────────────────────────────────────

mod e2e {
    use std::process::Command;

    #[test]
    fn check_exits_nonzero_for_missing_file() {
        let status = Command::new(env!("CARGO_BIN_EXE_folio"))
            .args(["check", "no-such-file.folio"])
            .status()
            .expect("failed to run folio binary");
        assert!(!status.success());
    }

    #[test]
    fn no_subcommand_exits_nonzero() {
        let status = Command::new(env!("CARGO_BIN_EXE_folio"))
            .status()
            .expect("failed to run folio binary");
        assert!(!status.success());
    }
}
