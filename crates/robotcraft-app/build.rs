fn main() {
    let epoch = ymd(0);
    if epoch != (1970, 1, 1) {
        panic!("unix epoch civil date was {epoch:?}");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../../CHANGELOG_LATEST.md");
    println!("cargo:rustc-env=ROBOTCRAFT_GIT_SHA={}", git_sha());
    println!("cargo:rustc-env=ROBOTCRAFT_BUILD_UTC={}", utc_stamp());
}

fn git_sha() -> String {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output();
    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    }
}

fn utc_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let tod = secs % 86_400;
    let hour = tod / 3_600;
    let min = (tod % 3_600) / 60;
    let (year, month, day) = ymd(days);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{min:02} UTC")
}

/// Howard Hinnant civil_from_days. `days` is days since 1970-01-01.
fn ymd(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}
