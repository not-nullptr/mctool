use std::{error::Error, fmt::Display, sync::LazyLock};

use mc_launchermeta::{VERSION_MANIFEST_URL, version::Version, version_manifest::Manifest};
use reqwest::Client;

static EXECUTABLE: LazyLock<String> = LazyLock::new(|| {
    fn failure() -> ! {
        eprintln!("failed to get executable name");
        std::process::exit(1)
    }

    std::env::args()
        .next()
        .unwrap_or_else(|| failure())
        .split("/")
        .last()
        .unwrap_or_else(|| failure())
        .to_string()
});

fn executable() -> &'static str {
    &EXECUTABLE
}

fn on_failure_errorless(msg: impl Display) -> ! {
    let executable = executable();
    eprintln!("{executable}: {msg}");
    std::process::exit(1);
}

fn on_failure(msg: impl Display, err: impl Display) -> ! {
    let executable = executable();
    eprintln!("{executable}: {msg}: {err}");
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let version = std::env::args()
        .nth(1)
        .unwrap_or_else(|| on_failure_errorless("first argument must be a minecraft version"));

    let client = Client::new();
    let manifest = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await
        .unwrap_or_else(|e| on_failure("failed to get the version list", e))
        .json::<Manifest>()
        .await
        .unwrap_or_else(|e| on_failure("failed to parse the version manifest api response", e));

    let version_url = &manifest
        .get_version(&version)
        .unwrap_or_else(|| on_failure("version not found", &version))
        .url;

    let version_info = client
        .get(version_url)
        .send()
        .await
        .unwrap_or_else(|e| on_failure(format!("failed to get {}", &version), e))
        .json::<Version>()
        .await
        .unwrap_or_else(|e| on_failure(format!("failed to parse the data for {}", &version), e));

    if let Some(ref download) = version_info.downloads.server {
        println!("{}", download.url);
    } else {
        on_failure("server doesn't exist for version", &version);
    }
}
