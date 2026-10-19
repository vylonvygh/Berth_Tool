use crate::error::{Error, Result};
use crate::types::Asset;

pub const SCORE_OS: i32 = 40;
pub const SCORE_ARCH: i32 = 40;
pub const SCORE_STATIC: i32 = 8;
pub const SCORE_NAME: i32 = 10;
pub const SCORE_FORMAT: i32 = 2;
pub const SCORE_THRESHOLD: i32 = 70;
pub const WIN_MARGIN: i32 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Os {
    Linux,
    Macos,
    Windows,
    Bsd,
    Solaris,
    Android,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arch {
    X86_64,
    Aarch64,
    Armv7,
    Armv6,
    Arm,
    X86_32,
    Riscv64,
    PowerPc64Le,
    S390x,
    LoongArch64,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

fn os_aliases(os: &Os) -> &'static [&'static str] {
    match os {
        Os::Linux => &["linux"],
        Os::Macos => &["darwin", "macos", "osx", "mac"],
        Os::Windows => &["windows", "win32", "win64", "msvc", "mingw"],
        Os::Bsd => &["freebsd", "openbsd", "netbsd", "dragonfly", "bsd"],
        Os::Solaris => &["solaris", "illumos", "sunos"],
        Os::Android => &["android"],
        Os::Other(_) => &[],
    }
}

fn arch_aliases(arch: &Arch) -> &'static [&'static str] {
    match arch {
        Arch::X86_64 => &["x86_64", "x86-64", "amd64", "x64"],
        Arch::Aarch64 => &["aarch64", "arm64", "armv8"],
        Arch::Armv7 => &["armv7", "armhf", "arm32v7"],
        Arch::Armv6 => &["armv6"],
        Arch::Arm => &["arm", "armel"],
        Arch::X86_32 => &["i686", "i586", "i486", "i386", "x86", "386"],
        Arch::Riscv64 => &["riscv64", "riscv"],
        Arch::PowerPc64Le => &["ppc64le", "powerpc64le", "ppc64el"],
        Arch::S390x => &["s390x"],
        Arch::LoongArch64 => &["loong64", "loongarch64"],
        Arch::Other(_) => &[],
    }
}

fn foreign_oses(own: &Os) -> &'static [&'static str] {
    match own {
        Os::Linux => &[
            "darwin",
            "macos",
            "osx",
            "windows",
            "win32",
            "win64",
            "msvc",
            "mingw",
            "freebsd",
            "openbsd",
            "netbsd",
            "dragonfly",
            "solaris",
            "illumos",
            "sunos",
            "android",
        ],
        Os::Macos => &[
            "linux", "windows", "win32", "win64", "msvc", "mingw", "android",
        ],
        Os::Windows => &["linux", "darwin", "macos", "osx", "android"],
        Os::Bsd => &[
            "linux", "darwin", "macos", "osx", "windows", "win32", "win64",
        ],
        Os::Solaris => &["linux", "darwin", "macos", "windows"],
        Os::Android => &["linux", "darwin", "macos", "windows"],
        Os::Other(_) => &[],
    }
}

fn foreign_arches(own: &Arch) -> &'static [&'static str] {
    match own {
        Arch::X86_64 => &[
            "aarch64",
            "arm64",
            "armv8",
            "armv7",
            "armhf",
            "arm32v7",
            "armv6",
            "armel",
            "i686",
            "i586",
            "i486",
            "i386",
            "riscv64",
            "ppc64le",
            "powerpc64le",
            "s390x",
            "loong64",
        ],
        Arch::Aarch64 => &[
            "x86_64", "x86-64", "amd64", "armv7", "armhf", "armv6", "i686", "i386", "riscv64",
            "ppc64le", "s390x", "loong64",
        ],
        Arch::Armv7 => &[
            "x86_64", "amd64", "aarch64", "arm64", "armv6", "i686", "riscv64",
        ],
        Arch::Armv6 => &["x86_64", "amd64", "aarch64", "arm64", "armv7", "i686"],
        Arch::Arm => &["x86_64", "amd64", "aarch64", "arm64", "i686"],
        Arch::X86_32 => &["x86_64", "x86-64", "amd64", "aarch64", "arm64", "riscv64"],
        Arch::Riscv64 => &["x86_64", "amd64", "aarch64", "arm64", "i686"],
        Arch::PowerPc64Le => &["x86_64", "amd64", "aarch64", "arm64"],
        Arch::S390x => &["x86_64", "amd64", "aarch64", "arm64"],
        Arch::LoongArch64 => &["x86_64", "amd64", "aarch64", "arm64"],
        Arch::Other(_) => &[],
    }
}

impl Arch {
    fn from_std(s: &str) -> Self {
        match s {
            "x86_64" => Arch::X86_64,
            "aarch64" => Arch::Aarch64,
            "arm" => Arch::Arm,
            "x86" => Arch::X86_32,
            "riscv64" => Arch::Riscv64,
            "powerpc64le" => Arch::PowerPc64Le,
            "s390x" => Arch::S390x,
            "loongarch64" => Arch::LoongArch64,
            other => Arch::Other(other.to_string()),
        }
    }
}

impl Os {
    fn from_std(s: &str) -> Self {
        match s {
            "linux" => Os::Linux,
            "macos" => Os::Macos,
            "windows" => Os::Windows,
            "freebsd" | "netbsd" | "openbsd" | "dragonfly" => Os::Bsd,
            "solaris" | "illumos" => Os::Solaris,
            "android" => Os::Android,
            other => Os::Other(other.to_string()),
        }
    }
}

impl Platform {
    pub fn current() -> Self {
        Self {
            os: Os::from_std(std::env::consts::OS),
            arch: Arch::from_std(std::env::consts::ARCH),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exclusion {
    Checksum,
    Package,
    SourceArchive,
    ForeignOs,
    ForeignArch,
}

fn tokens(name_lower: &str) -> Vec<&str> {
    name_lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect()
}

fn alias_present(name_lower: &str, alias: &str, token_set: &[&str]) -> bool {
    if alias.len() <= 4 {
        token_set.contains(&alias)
    } else {
        name_lower.contains(alias)
    }
}

fn ends_with_any(name_lower: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|s| name_lower.ends_with(s))
}

pub fn exclusion_reason(name: &str, platform: &Platform) -> Option<Exclusion> {
    let lower = name.to_ascii_lowercase();
    if ends_with_any(
        &lower,
        &[
            ".sha256",
            ".sha512",
            ".sig",
            ".asc",
            ".pem",
            ".sbom",
            ".intoto.jsonl",
        ],
    ) {
        return Some(Exclusion::Checksum);
    }
    if ends_with_any(
        &lower,
        &[
            ".deb",
            ".rpm",
            ".apk",
            ".msi",
            ".exe",
            ".dmg",
            ".pkg",
            ".appimage",
        ],
    ) {
        return Some(Exclusion::Package);
    }
    let toks = tokens(&lower);
    if toks.contains(&"source") || toks.contains(&"src") {
        return Some(Exclusion::SourceArchive);
    }

    let own_os = os_aliases(&platform.os);
    let has_own_os = own_os.iter().any(|a| alias_present(&lower, a, &toks));
    if !has_own_os {
        for alias in foreign_oses(&platform.os) {
            if alias_present(&lower, alias, &toks) {
                return Some(Exclusion::ForeignOs);
            }
        }
    }

    let own_arch = arch_aliases(&platform.arch);
    let has_own_arch = own_arch.iter().any(|a| alias_present(&lower, a, &toks));
    if !has_own_arch {
        for alias in foreign_arches(&platform.arch) {
            if alias_present(&lower, alias, &toks) {
                return Some(Exclusion::ForeignArch);
            }
        }
    }

    None
}

fn format_accepted(lower: &str) -> bool {
    ends_with_any(
        lower,
        &[
            ".tar.gz", ".tgz", ".tar.xz", ".txz", ".tar.zst", ".tzst", ".tar.bz2", ".tbz2", ".zip",
        ],
    )
}

pub fn score_asset(asset: &Asset, platform: &Platform, package: &str) -> i32 {
    let lower = asset.name.to_ascii_lowercase();
    let toks = tokens(&lower);
    let mut score = 0;

    if os_aliases(&platform.os)
        .iter()
        .any(|a| alias_present(&lower, a, &toks))
    {
        score += SCORE_OS;
    }
    if arch_aliases(&platform.arch)
        .iter()
        .any(|a| alias_present(&lower, a, &toks))
    {
        score += SCORE_ARCH;
    }
    if lower.contains("musl") || lower.contains("static") {
        score += SCORE_STATIC;
    }
    if !package.is_empty() && lower.contains(&package.to_ascii_lowercase()) {
        score += SCORE_NAME;
    }
    if format_accepted(&lower) {
        score += SCORE_FORMAT;
    }
    score
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedAsset<'a> {
    pub asset: &'a Asset,
    pub score: i32,
}

fn rank_refs<'a>(
    assets: Vec<&'a Asset>,
    platform: &Platform,
    package: &str,
) -> Vec<RankedAsset<'a>> {
    let mut ranked: Vec<RankedAsset<'a>> = assets
        .into_iter()
        .filter(|a| exclusion_reason(&a.name, platform).is_none())
        .map(|asset| RankedAsset {
            asset,
            score: score_asset(asset, platform, package),
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.asset.name.cmp(&b.asset.name))
    });
    ranked
}

pub fn rank_assets<'a>(
    assets: &'a [Asset],
    platform: &Platform,
    package: &str,
) -> Vec<RankedAsset<'a>> {
    rank_refs(assets.iter().collect(), platform, package)
}

#[derive(Debug, Clone, Default)]
pub struct SelectOpts {
    pub interactive: bool,
    pub yes: bool,
    pub asset_regex: Option<String>,
}

#[derive(Debug)]
pub enum Selection<'a> {
    Chosen(RankedAsset<'a>),
    Candidates(Vec<RankedAsset<'a>>),
}

fn filter_by_regex<'a>(assets: &'a [Asset], pattern: &str) -> Result<Vec<&'a Asset>> {
    let re = regex::Regex::new(pattern).map_err(|e| Error::Parse {
        what: format!("asset regex {pattern:?}"),
        message: e.to_string(),
    })?;
    let matched: Vec<&'a Asset> = assets.iter().filter(|a| re.is_match(&a.name)).collect();
    if matched.is_empty() {
        return Err(Error::NoMatch(format!("no assets match regex {pattern:?}")));
    }
    Ok(matched)
}

pub fn select_asset<'a>(
    assets: &'a [Asset],
    platform: &Platform,
    package: &str,
    opts: &SelectOpts,
) -> Result<Selection<'a>> {
    let explicit = opts.asset_regex.is_some();
    let ranked = match &opts.asset_regex {
        Some(pattern) => rank_refs(filter_by_regex(assets, pattern)?, platform, package),
        None => rank_refs(assets.iter().collect(), platform, package),
    };

    let Some(top) = ranked.first() else {
        return Err(Error::NoMatch(
            "no assets match this platform and package".to_string(),
        ));
    };
    let second = ranked.get(1);
    let margin_ok = match second {
        None => true,
        Some(s) => top.score - s.score >= WIN_MARGIN,
    };
    let confident = top.score >= SCORE_THRESHOLD && margin_ok;
    let forced = opts.yes && margin_ok;
    let single_explicit = explicit && ranked.len() == 1;

    if confident || forced || single_explicit {
        Ok(Selection::Chosen(top.clone()))
    } else {
        Ok(Selection::Candidates(ranked))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub path: String,
    pub is_dir: bool,
    pub executable: Option<bool>,
}

fn skip_stem(stem_lower: &str) -> bool {
    matches!(
        stem_lower,
        "readme"
            | "license"
            | "licensing"
            | "changelog"
            | "contributing"
            | "notice"
            | "authors"
            | "codeowners"
            | "makefile"
            | "dockerfile"
            | "cargo.toml"
            | "cargo.lock"
            | "go.mod"
            | "package.json"
            | "copying"
            | "manifest"
    )
}

fn skip_ext(ext_lower: &str) -> bool {
    matches!(
        ext_lower,
        "md" | "txt"
            | "1"
            | "2"
            | "3"
            | "html"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "fish"
            | "zsh"
            | "bash"
            | "sh"
            | "ps1"
            | "bat"
            | "cmd"
            | "so"
            | "dylib"
            | "dll"
            | "a"
            | "o"
            | "lib"
            | "gz"
            | "xz"
            | "zst"
            | "bz2"
            | "zip"
            | "tar"
    )
}

fn skip_component(c_lower: &str) -> bool {
    matches!(
        c_lower,
        "man"
            | "docs"
            | "doc"
            | "share"
            | "lib"
            | "lib64"
            | "include"
            | "etc"
            | "completions"
            | "contrib"
            | "examples"
            | "src"
            | "locale"
    )
}

pub fn is_probable_executable(entry: &ArchiveEntry) -> bool {
    if entry.is_dir {
        return false;
    }
    let path = entry.path.to_ascii_lowercase();
    let file = path.rsplit('/').next().unwrap_or(&path);
    let (stem, ext) = match file.rsplit_once('.') {
        Some((s, e)) => (s, e),
        None => (file, ""),
    };
    if skip_stem(stem) || skip_ext(ext) {
        return false;
    }
    path.split('/').rev().skip(1).all(|c| !skip_component(c))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedEntry<'a> {
    pub entry: &'a ArchiveEntry,
    pub score: i32,
}

fn entry_score(entry: &ArchiveEntry, package: &str) -> i32 {
    let lower = entry.path.to_ascii_lowercase();
    let file = lower.rsplit('/').next().unwrap_or(&lower);
    let stem = file.split('.').next().unwrap_or(file);
    let mut score: i32 = 0;
    if stem == package.to_ascii_lowercase() {
        score += 100;
    } else if stem.contains(&package.to_ascii_lowercase()) {
        score += 30;
    }
    match entry.executable {
        Some(true) => score += 20,
        Some(false) => score -= 10,
        None => score += 5,
    }
    let depth = lower.matches('/').count();
    if depth == 0 {
        score += 30;
    } else {
        let first = lower.split('/').next().unwrap_or("");
        if matches!(first, "bin" | "sbin") || lower.starts_with("usr/bin/") {
            score += 25;
        }
        score -= depth.min(5) as i32;
    }
    score
}

pub fn discover_executables<'a>(
    entries: &'a [ArchiveEntry],
    package: &str,
) -> Vec<RankedEntry<'a>> {
    let candidates: Vec<&'a ArchiveEntry> = entries
        .iter()
        .filter(|e| is_probable_executable(e))
        .collect();
    let any_exec = candidates.iter().any(|e| e.executable == Some(true));
    let filtered: Vec<&'a ArchiveEntry> = candidates
        .into_iter()
        .filter(|e| !any_exec || e.executable != Some(false))
        .collect();
    let mut ranked: Vec<RankedEntry<'a>> = filtered
        .into_iter()
        .map(|entry| RankedEntry {
            entry,
            score: entry_score(entry, package),
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| {
                let da = a.entry.path.matches('/').count();
                let db = b.entry.path.matches('/').count();
                da.cmp(&db)
            })
            .then_with(|| a.entry.path.cmp(&b.entry.path))
    });
    ranked
}

pub fn choose_executable<'a>(
    entries: &'a [ArchiveEntry],
    package: &str,
) -> Option<&'a ArchiveEntry> {
    discover_executables(entries, package)
        .first()
        .map(|r| r.entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn asset(name: &str) -> Asset {
        Asset {
            name: name.to_string(),
            url: Url::parse(&format!("https://example.com/{name}")).unwrap(),
            size: None,
            digest: None,
        }
    }

    fn linux_x86() -> Platform {
        Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        }
    }

    fn names(ranked: &[RankedAsset]) -> Vec<String> {
        ranked.iter().map(|r| r.asset.name.clone()).collect()
    }

    #[test]
    fn excludes_checksums() {
        let p = linux_x86();
        for name in [
            "fzf-0.74.5-linux_amd64.tar.gz.sha256",
            "fzf-0.74.5-linux_amd64.tar.gz.sha512",
            "fzf.sig",
            "fzf.asc",
            "cert.pem",
            "sbom.intoto.jsonl",
        ] {
            assert_eq!(
                exclusion_reason(name, &p),
                Some(Exclusion::Checksum),
                "{name}"
            );
        }
    }

    #[test]
    fn excludes_foreign_packages() {
        let p = linux_x86();
        for name in [
            "tool_1.0_amd64.deb",
            "tool-1.0.x86_64.rpm",
            "tool.apk",
            "tool.msi",
            "tool.exe",
            "tool.dmg",
            "tool.pkg",
            "tool.AppImage",
        ] {
            assert_eq!(
                exclusion_reason(name, &p),
                Some(Exclusion::Package),
                "{name}"
            );
        }
    }

    #[test]
    fn excludes_source_archives() {
        let p = linux_x86();
        for name in [
            "fzf-0.74.5-source.tar.gz",
            "fzf-0.74.5-src.tar.gz",
            "fzf_src.zip",
            "fzf-src.tgz",
        ] {
            assert_eq!(
                exclusion_reason(name, &p),
                Some(Exclusion::SourceArchive),
                "{name}"
            );
        }
        assert_eq!(exclusion_reason("fzf-0.74.5-linux_amd64.tar.gz", &p), None);
    }

    #[test]
    fn excludes_foreign_os() {
        let p = linux_x86();
        for name in [
            "fzf-0.74.5-darwin_amd64.tar.gz",
            "fzf-0.74.5-macos_arm64.tar.gz",
            "fzf-0.74.5-windows_amd64.zip",
            "fzf-0.74.5-win64.zip",
            "fzf-0.74.5-freebsd_amd64.tar.gz",
            "fzf-0.74.5-android_arm64.tar.gz",
        ] {
            assert_eq!(
                exclusion_reason(name, &p),
                Some(Exclusion::ForeignOs),
                "{name}"
            );
        }
    }

    #[test]
    fn excludes_foreign_arch() {
        let p = linux_x86();
        for name in [
            "fzf-0.74.5-linux_arm64.tar.gz",
            "fzf-0.74.5-linux_armv7.tar.gz",
            "fzf-0.74.5-linux_i686.tar.gz",
            "fzf-0.74.5-linux_riscv64.tar.gz",
        ] {
            assert_eq!(
                exclusion_reason(name, &p),
                Some(Exclusion::ForeignArch),
                "{name}"
            );
        }
    }

    #[test]
    fn keeps_asset_that_mentions_own_os_and_arch() {
        let p = linux_x86();
        assert_eq!(exclusion_reason("fzf-0.74.5-linux_amd64.tar.gz", &p), None);
        assert_eq!(
            exclusion_reason("fzf-0.74.5-x86_64-unknown-linux-musl.tar.gz", &p),
            None
        );
        assert_eq!(
            exclusion_reason("ripgrep-14.1.0-x86_64-unknown-linux-gnu.tar.gz", &p),
            None
        );
    }

    #[test]
    fn musl_and_name_and_format_scores() {
        let p = linux_x86();
        let musl = asset("fd-v9.0.0-x86_64-unknown-linux-musl.tar.gz");
        let gnu = asset("fd-v9.0.0-x86_64-unknown-linux-gnu.tar.gz");
        let sm = score_asset(&musl, &p, "fd");
        let gs = score_asset(&gnu, &p, "fd");
        assert_eq!(
            sm,
            SCORE_OS + SCORE_ARCH + SCORE_STATIC + SCORE_NAME + SCORE_FORMAT
        );
        assert_eq!(gs, SCORE_OS + SCORE_ARCH + SCORE_NAME + SCORE_FORMAT);
        assert_eq!(sm - gs, SCORE_STATIC);
    }

    #[test]
    fn ranking_excludes_and_orders_by_score() {
        let p = linux_x86();
        let assets = vec![
            asset("tool-1.0-darwin_amd64.tar.gz"),
            asset("tool-1.0-linux_arm64.tar.gz"),
            asset("tool-1.0-linux_amd64.tar.gz"),
            asset("tool-1.0-linux_amd64.tar.gz.sha256"),
            asset("tool-1.0-linux_amd64.deb"),
        ];
        let ranked = rank_assets(&assets, &p, "tool");
        assert_eq!(
            names(&ranked),
            vec!["tool-1.0-linux_amd64.tar.gz".to_string()]
        );
    }

    #[test]
    fn selection_is_confident_for_single_platform_match() {
        let p = linux_x86();
        let assets = vec![
            asset("rg-14.1.0-x86_64-unknown-linux-gnu.tar.gz"),
            asset("rg-14.1.0-aarch64-unknown-linux-gnu.tar.gz"),
        ];
        let sel = select_asset(&assets, &p, "ripgrep", &SelectOpts::default()).unwrap();
        match sel {
            Selection::Chosen(r) => {
                assert_eq!(r.asset.name, "rg-14.1.0-x86_64-unknown-linux-gnu.tar.gz");
                assert!(r.score >= SCORE_THRESHOLD);
            }
            Selection::Candidates(_) => panic!("expected confident choice"),
        }
    }

    #[test]
    fn tie_is_ambiguous() {
        let p = linux_x86();
        let assets = vec![
            asset("tool-1.0-linux-a.tar.gz"),
            asset("tool-1.0-linux-b.tar.gz"),
        ];
        let ranked = rank_assets(&assets, &p, "tool");
        assert_eq!(ranked[0].score, ranked[1].score);
        let sel = select_asset(&assets, &p, "tool", &SelectOpts::default()).unwrap();
        assert!(matches!(sel, Selection::Candidates(_)));
    }

    #[test]
    fn musl_wins_with_margin() {
        let p = linux_x86();
        let assets = vec![
            asset("tool-1.0-x86_64-linux-gnu.tar.gz"),
            asset("tool-1.0-x86_64-linux-musl.tar.gz"),
        ];
        let sel = select_asset(&assets, &p, "tool", &SelectOpts::default()).unwrap();
        match sel {
            Selection::Chosen(r) => assert!(r.asset.name.contains("musl")),
            Selection::Candidates(_) => panic!("musl should win on margin"),
        }
    }

    #[test]
    fn below_threshold_without_match_is_ambiguous() {
        let p = linux_x86();
        let assets = vec![asset("mystery-build.tar.gz")];
        let sel = select_asset(&assets, &p, "tool", &SelectOpts::default()).unwrap();
        assert!(matches!(sel, Selection::Candidates(_)));
    }

    #[test]
    fn yes_picks_single_candidate_below_threshold() {
        let p = linux_x86();
        let assets = vec![asset("mystery-build.tar.gz")];
        let opts = SelectOpts {
            yes: true,
            ..Default::default()
        };
        let sel = select_asset(&assets, &p, "tool", &opts).unwrap();
        assert!(matches!(sel, Selection::Chosen(_)));
    }

    #[test]
    fn yes_does_not_break_tie() {
        let p = linux_x86();
        let assets = vec![
            asset("tool-1.0-linux-musl.tar.gz"),
            asset("tool-1.0-linux-musl-alt.tar.gz"),
        ];
        let opts = SelectOpts {
            yes: true,
            ..Default::default()
        };
        let sel = select_asset(&assets, &p, "tool", &opts).unwrap();
        assert!(matches!(sel, Selection::Candidates(_)));
    }

    #[test]
    fn no_matching_assets_is_an_error() {
        let p = linux_x86();
        let assets = vec![asset("tool-1.0-darwin_amd64.tar.gz")];
        let err = select_asset(&assets, &p, "tool", &SelectOpts::default()).unwrap_err();
        assert!(matches!(err, Error::NoMatch(_)));
    }

    #[test]
    fn asset_regex_overrides_scoring() {
        let p = linux_x86();
        let assets = vec![
            asset("tool-1.0-x86_64-linux-gnu.tar.gz"),
            asset("tool-1.0-x86_64-linux-musl.tar.gz"),
        ];
        let opts = SelectOpts {
            asset_regex: Some("gnu".to_string()),
            ..Default::default()
        };
        let sel = select_asset(&assets, &p, "tool", &opts).unwrap();
        match sel {
            Selection::Chosen(r) => assert_eq!(r.asset.name, "tool-1.0-x86_64-linux-gnu.tar.gz"),
            Selection::Candidates(_) => panic!("regex single match should be chosen"),
        }
    }

    #[test]
    fn asset_regex_ignores_excluded_platform() {
        let p = linux_x86();
        let assets = vec![asset("tool-1.0-darwin_amd64.tar.gz")];
        let opts = SelectOpts {
            asset_regex: Some("darwin".to_string()),
            ..Default::default()
        };
        let err = select_asset(&assets, &p, "tool", &opts).unwrap_err();
        assert!(matches!(err, Error::NoMatch(_)));
    }

    #[test]
    fn invalid_asset_regex_is_an_error() {
        let p = linux_x86();
        let assets = vec![asset("tool-1.0-linux_amd64.tar.gz")];
        let opts = SelectOpts {
            asset_regex: Some("[".to_string()),
            ..Default::default()
        };
        let err = select_asset(&assets, &p, "tool", &opts).unwrap_err();
        assert!(matches!(err, Error::Parse { .. }));
    }

    #[test]
    fn no_match_after_regex_is_an_error() {
        let p = linux_x86();
        let assets = vec![asset("tool-1.0-linux_amd64.tar.gz")];
        let opts = SelectOpts {
            asset_regex: Some("riscv".to_string()),
            ..Default::default()
        };
        let err = select_asset(&assets, &p, "tool", &opts).unwrap_err();
        assert!(matches!(err, Error::NoMatch(_)));
    }

    fn entry(path: &str, executable: Option<bool>) -> ArchiveEntry {
        ArchiveEntry {
            path: path.to_string(),
            is_dir: false,
            executable,
        }
    }

    #[test]
    fn prefers_package_named_binary_in_root() {
        let entries = vec![
            entry("README.md", Some(true)),
            entry("LICENSE", None),
            entry("bin/helper", Some(true)),
            entry("fzf", Some(true)),
        ];
        let picked = choose_executable(&entries, "fzf").unwrap();
        assert_eq!(picked.path, "fzf");
    }

    #[test]
    fn prefers_bin_directory() {
        let entries = vec![
            entry("share/doc/data.txt", Some(true)),
            entry("bin/tool", Some(true)),
            entry("opt/nested/helper", Some(true)),
        ];
        let picked = choose_executable(&entries, "other").unwrap();
        assert_eq!(picked.path, "bin/tool");
    }

    #[test]
    fn skips_docs_libraries_and_scripts() {
        let entries = vec![
            entry("README.md", Some(true)),
            entry("CHANGELOG.md", Some(true)),
            entry("completions/fzf.bash", Some(true)),
            entry("lib/libfoo.so", Some(true)),
            entry("share/man/man1/fzf.1", Some(true)),
            entry("install.sh", Some(true)),
        ];
        assert!(choose_executable(&entries, "fzf").is_none());
    }

    #[test]
    fn skips_non_executables_when_executables_exist() {
        let entries = vec![entry("tool", Some(false)), entry("bin/tool", Some(true))];
        let picked = choose_executable(&entries, "tool").unwrap();
        assert_eq!(picked.path, "bin/tool");
    }

    #[test]
    fn accepts_unknown_mode_when_name_matches() {
        let entries = vec![entry("subdir/other", None), entry("gadget", None)];
        let picked = choose_executable(&entries, "gadget").unwrap();
        assert_eq!(picked.path, "gadget");
    }

    #[test]
    fn nested_top_dir_binary_is_found() {
        let entries = vec![
            entry("fzf-0.74.5-linux_amd64/README.md", Some(false)),
            entry("fzf-0.74.5-linux_amd64/fzf", Some(true)),
        ];
        let picked = choose_executable(&entries, "fzf").unwrap();
        assert_eq!(picked.path, "fzf-0.74.5-linux_amd64/fzf");
    }

    #[test]
    fn directories_are_never_candidates() {
        let entries = vec![ArchiveEntry {
            path: "bin".to_string(),
            is_dir: true,
            executable: Some(true),
        }];
        assert!(choose_executable(&entries, "bin").is_none());
    }

    #[test]
    fn discovery_is_deterministic() {
        let entries = vec![
            entry("bin/a", Some(true)),
            entry("bin/b", Some(true)),
            entry("opt/c", Some(true)),
        ];
        let first = discover_executables(&entries, "x");
        let second = discover_executables(&entries, "x");
        assert_eq!(first, second);
        assert_eq!(first.len(), 3);
    }

    #[test]
    fn current_platform_is_usable() {
        let p = Platform::current();
        let assets = vec![
            asset("tool-1.0-linux_amd64.tar.gz"),
            asset("tool-1.0-darwin_arm64.tar.gz"),
        ];
        let ranked = rank_assets(&assets, &p, "tool");
        if p.os == Os::Linux && p.arch == Arch::X86_64 {
            assert_eq!(ranked.len(), 1);
            assert_eq!(ranked[0].asset.name, "tool-1.0-linux_amd64.tar.gz");
        } else {
            assert!(!ranked.is_empty() || p.os != Os::Linux);
        }
    }

    #[test]
    fn case_insensitive_name_bonus() {
        let p = linux_x86();
        let a = asset("Btop-4.0.0-x86_64-linux.tar.gz");
        let b = asset("btop-4.0.0-x86_64-linux.tar.gz");
        assert_eq!(score_asset(&a, &p, "btop"), score_asset(&b, &p, "btop"));
    }
}
