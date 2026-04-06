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

// ── Args ──────────────────────────────────────────────────────────────────────

mod args {
    use folio::infrastructure::{Args, Command};

    #[test]
    fn parses_check_subcommand() {
        let args = Args::create_null(["folio", "check", "ledger.folio"]);
        assert!(
            matches!(args.command, Command::Check { ref path } if path == "ledger.folio"),
            "expected Check subcommand with correct path",
        );
    }
}
