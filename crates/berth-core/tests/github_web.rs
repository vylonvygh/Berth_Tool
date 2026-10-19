use berth_core::github::web::GitHubWeb;

#[test]
fn parse_expanded_assets_fzf() {
    let html =
        std::fs::read_to_string("../../tests/fixtures/github/fzf-expanded_assets.html").unwrap();
    let rows = GitHubWeb::parse_expanded_assets(&html);
    assert!(rows.len() > 10);
    // find linux_amd64
    let amd64 = rows
        .iter()
        .find(|r| r.name.contains("linux_amd64"))
        .unwrap();
    assert!(amd64.url_relative.contains("/releases/download/"));
    assert!(amd64.sha256.is_some());
    assert!(amd64.size.is_some());
}
