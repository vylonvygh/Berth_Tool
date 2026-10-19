use berth_core::github::web::{
    header_value, latest_tag_from_location, tag_from_link, version_from_tag, GitHubWeb,
};
use berth_core::provider::{Provider, ResolveOpts};
use berth_core::types::{ProviderId, RepoRef, Version};

fn fzf_repo() -> RepoRef {
    RepoRef {
        provider: ProviderId::GitHub,
        host: "github.com".to_string(),
        owner: "junegunn".to_string(),
        repo: "fzf".to_string(),
    }
}

fn fixture(path: &str) -> String {
    std::fs::read_to_string(format!("../../tests/fixtures/github/{path}"))
        .unwrap_or_else(|e| panic!("read fixture {path}: {e}"))
}

#[test]
fn parse_expanded_assets_fzf() {
    let rows = GitHubWeb::parse_expanded_assets(&fixture("fzf-expanded_assets.html")).unwrap();
    assert_eq!(rows.len(), 26);
    assert_eq!(rows[0].name, "fzf-0.74.5-android_arm64.tar.gz");
    assert_eq!(
        rows[0].url_relative,
        "/junegunn/fzf/releases/download/v0.74.5/fzf-0.74.5-android_arm64.tar.gz"
    );
    assert!(rows[0].size.is_some());
    assert!(rows[0].published.is_some());

    let amd64 = rows
        .iter()
        .find(|r| r.name.contains("linux_amd64"))
        .expect("linux_amd64 asset");
    assert_eq!(amd64.size, Some((2.18 * 1024.0 * 1024.0) as u64));
    assert_eq!(
        amd64.sha256.as_deref(),
        Some("d47d7f882972c7d674dbbba998ffa4116e68379616ffac7695d0671e9a722c5f")
    );
    assert!(rows.iter().all(|r| r.sha256.is_some()));
    assert!(rows.iter().all(|r| r.size.is_some()));
}

#[test]
fn parse_expanded_assets_rejects_empty() {
    let err = GitHubWeb::parse_expanded_assets("<html><body>nothing</body></html>").unwrap_err();
    assert!(err.to_string().contains("no release download links"));
}

#[test]
fn rows_to_assets_builds_absolute_urls() {
    let rows = GitHubWeb::parse_expanded_assets(&fixture("fzf-expanded_assets.html")).unwrap();
    let assets = berth_core::github::web::rows_to_assets(&rows, "github.com").unwrap();
    assert_eq!(assets.len(), 26);
    let amd64 = assets
        .iter()
        .find(|a| a.name.contains("linux_amd64"))
        .unwrap();
    assert_eq!(
        amd64.url.as_str(),
        "https://github.com/junegunn/fzf/releases/download/v0.74.5/fzf-0.74.5-linux_amd64.tar.gz"
    );
    let digest = amd64.digest.as_ref().unwrap();
    assert_eq!(digest.algorithm, "sha256");
    assert_eq!(digest.hex.len(), 64);
}

#[test]
fn latest_tag_from_fixture_headers() {
    let raw = fixture("fzf-releases-latest.headers");
    let location = header_value(&raw, "location").expect("location header");
    let tag = latest_tag_from_location(&location, &fzf_repo()).unwrap();
    assert_eq!(tag, "v0.74.5");

    let raw = fixture("linux-releases-latest.headers");
    let location = header_value(&raw, "location").unwrap();
    let err = latest_tag_from_location(
        &location,
        &RepoRef {
            provider: ProviderId::GitHub,
            host: "github.com".to_string(),
            owner: "torvalds".to_string(),
            repo: "linux".to_string(),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("no releases published"));
}

#[test]
fn rejects_insecure_redirect_location() {
    let err = latest_tag_from_location(
        "http://github.com/junegunn/fzf/releases/tag/v1.0.0",
        &fzf_repo(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("insecure"));
}

#[test]
fn header_value_is_case_insensitive() {
    let raw = "HTTP/2 302 \r\nLocation: https://example.com/x\r\ncontent-length: 0\r\n";
    assert_eq!(
        header_value(raw, "location").as_deref(),
        Some("https://example.com/x")
    );
    assert_eq!(header_value(raw, "x-missing"), None);
}

#[test]
fn tag_from_link_decodes_percent_encoding() {
    assert_eq!(
        tag_from_link("https://github.com/o/r/releases/tag/v1.0.0%2Bbuild"),
        Some("v1.0.0+build".to_string())
    );
    assert_eq!(tag_from_link("https://github.com/o/r/releases"), None);
}

#[test]
fn version_from_tag_parses_semver() {
    assert!(matches!(
        version_from_tag("v0.74.5"),
        Version::Semver(v) if v.to_string() == "0.74.5"
    ));
    assert!(matches!(
        version_from_tag("nightly-2026-10-11"),
        Version::Tag(t) if t == "nightly-2026-10-11"
    ));
}

#[tokio::test]
async fn latest_follows_redirect_and_loads_assets() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = format!("http://{}", server.address());
    let location = format!("{base}/junegunn/fzf/releases/tag/v0.74.5");
    Mock::given(method("GET"))
        .and(path("/junegunn/fzf/releases/latest"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", location.as_str()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/junegunn/fzf/releases/tag/v0.74.5"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html></html>"))
        .mount(&server)
        .await;
    let html = fixture("fzf-expanded_assets.html");
    Mock::given(method("GET"))
        .and(path("/junegunn/fzf/releases/expanded_assets/v0.74.5"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&server)
        .await;

    let provider = GitHubWeb::with_base("berth-test", base).unwrap();
    let release = provider
        .latest(&fzf_repo(), &ResolveOpts::default())
        .await
        .unwrap();
    assert_eq!(release.tag, "v0.74.5");
    assert!(matches!(&release.version, Version::Semver(v) if v.to_string() == "0.74.5"));
    assert_eq!(release.assets.len(), 26);
    assert!(release.published.is_some());
}

#[tokio::test]
async fn list_releases_from_atom() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = format!("http://{}", server.address());
    let xml = fixture("fzf-releases.atom");
    Mock::given(method("GET"))
        .and(path("/junegunn/fzf/releases.atom"))
        .respond_with(ResponseTemplate::new(200).set_body_string(xml))
        .mount(&server)
        .await;

    let provider = GitHubWeb::with_base("berth-test", base).unwrap();
    let list = provider.list_releases(&fzf_repo(), 3).await.unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].tag, "v0.74.5");
    assert!(list[0].published.is_some());
    assert!(matches!(&list[0].version, Version::Semver(_)));
}

#[tokio::test]
async fn http_error_is_reported() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = format!("http://{}", server.address());
    Mock::given(method("GET"))
        .and(path("/junegunn/fzf/releases/expanded_assets/v9.9.9"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let provider = GitHubWeb::with_base("berth-test", base).unwrap();
    let err = provider.release(&fzf_repo(), "v9.9.9").await.unwrap_err();
    assert!(err.to_string().contains("HTTP request"));
}
