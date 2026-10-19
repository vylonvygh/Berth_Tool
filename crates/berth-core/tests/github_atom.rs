use berth_core::github::web::{summary_from_entry, GitHubWeb};

fn fixture(path: &str) -> String {
    std::fs::read_to_string(format!("../../tests/fixtures/github/{path}"))
        .unwrap_or_else(|e| panic!("read fixture {path}: {e}"))
}

#[test]
fn parse_fzf_atom() {
    let entries = GitHubWeb::parse_atom(&fixture("fzf-releases.atom")).unwrap();
    assert_eq!(entries.len(), 10);
    assert_eq!(entries[0].tag().as_deref(), Some("v0.74.5"));
    assert_eq!(entries[0].title, "0.74.5");
    assert!(entries[0].updated.is_some());

    let summary = summary_from_entry(&entries[0]).unwrap();
    assert_eq!(summary.tag, "v0.74.5");
    assert!(!summary.prerelease);
    assert!(summary.published.is_some());
    assert!(
        matches!(&summary.version, berth_core::Version::Semver(v) if v.to_string() == "0.74.5")
    );
}

#[test]
fn parse_neovim_atom() {
    let entries = GitHubWeb::parse_atom(&fixture("neovim-releases.atom")).unwrap();
    assert_eq!(entries.len(), 10);
    assert_eq!(entries[0].title, "Nvim development (prerelease) build");
    assert!(entries[0].tag().is_some());
    let summary = summary_from_entry(&entries[0]).unwrap();
    assert!(summary.published.is_some());
}

#[test]
fn parse_atom_rejects_malformed() {
    let err = GitHubWeb::parse_atom("<feed><entry>").unwrap_err();
    assert!(err.to_string().contains("atom feed"));
}

#[test]
fn summary_requires_a_tag() {
    let entries = GitHubWeb::parse_atom(&fixture("fzf-releases.atom")).unwrap();
    let mut entry = entries[0].clone();
    entry.link = Some("https://github.com/junegunn/fzf/releases".to_string());
    let err = summary_from_entry(&entry).unwrap_err();
    assert!(err.to_string().contains("no release tag"));
}
