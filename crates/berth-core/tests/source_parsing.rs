use berth_core::source::SourceSpec;
use berth_core::ProviderId;

#[test]
fn parse_shorthand_junegunn_fzf() {
    let s = SourceSpec::from_input("junegunn/fzf").unwrap();
    assert_eq!(s.repo.provider, ProviderId::GitHub);
    assert_eq!(s.repo.host, "github.com");
    assert_eq!(s.repo.owner, "junegunn");
    assert_eq!(s.repo.repo, "fzf");
    assert!(s.version.is_none());
}

#[test]
fn parse_with_scheme() {
    let s = SourceSpec::from_input("https://github.com/junegunn/fzf").unwrap();
    assert_eq!(s.repo.owner, "junegunn");
    assert_eq!(s.repo.repo, "fzf");
}

#[test]
fn parse_without_scheme() {
    let s = SourceSpec::from_input("github.com/junegunn/fzf").unwrap();
    assert_eq!(s.repo.owner, "junegunn");
    assert_eq!(s.repo.repo, "fzf");
}

#[test]
fn parse_release_url() {
    let s = SourceSpec::from_input("https://github.com/junegunn/fzf/releases/tag/v0.55.0").unwrap();
    assert_eq!(s.repo.owner, "junegunn");
    assert_eq!(s.repo.repo, "fzf");
}

#[test]
fn parse_explicit_provider() {
    let s = SourceSpec::from_input("gitlab:group/project").unwrap();
    assert_eq!(s.repo.provider, ProviderId::GitLab);
    assert_eq!(s.repo.host, "gitlab.com");
}

#[test]
fn parse_full_gitlab_url() {
    let s = SourceSpec::from_input("https://gitlab.com/group/project").unwrap();
    assert_eq!(s.repo.provider, ProviderId::GitLab);
    assert_eq!(s.repo.owner, "group");
    assert_eq!(s.repo.repo, "project");
}

#[test]
fn parse_at_version() {
    let s = SourceSpec::from_input("junegunn/fzf@0.55.0").unwrap();
    assert_eq!(s.version.as_deref(), Some("0.55.0"));
}
