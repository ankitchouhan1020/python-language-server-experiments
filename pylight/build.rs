use chrono::Utc;

fn main() {
    // Generate build timestamp
    let build_time = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={build_time}");

    // Also include git commit hash if available
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=GIT_COMMIT_HASH={git_hash}");
    } else {
        println!("cargo:rustc-env=GIT_COMMIT_HASH=unknown");
    }

    // Force rebuild if any source file changes
    println!("cargo:rerun-if-changed=src/");
}
