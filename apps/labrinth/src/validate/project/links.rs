mod description;

use std::collections::HashMap;

use serde_json::json;
use url::Url;

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::{
    projects::{Project, Version},
    v2::projects::LegacyProject,
};

const GLOBAL_BLOCKS: &[&str] = &[
    "bit.ly",
    "adf.ly",
    "tinyurl.com",
    "short.io",
    "is.gd",
    "t.me",
    "telegram.org",
    "linkvertise.com",
    "example.com",
    "example.net",
    "example.org",
    "9minecraft.net",
    "creativemode.net",
    "orcaclient.com",
    "autoforged.cn",
];
const EXTERNAL_BLOCKS: &[&str] = &[
    "reddit.com",
    "redd.it",
    "x.com",
    "twitter.com",
    "instagram.com",
    "facebook.com",
    "tiktok.com",
    "twitch.tv",
    "bsky.app",
    "bilibili.com",
    "youtube.com",
    "youtu.be",
    "curseforge.com",
    "planetminecraft.com",
    "modrinth.com",
    "minecraft.net",
    "mcmod.cn",
    "github.com",
    "drive.google.com",
    "www.google.com",
    "google.com",
    "dropbox.com",
    "mediafire.com",
    "linktr.ee",
    "amazon.com",
    "discord.com",
    "discord.gg",
    "discordapp.com",
];
const SOURCE_DOMAINS: &[&str] = &[
    "github.com",
    "gitlab.com",
    "bitbucket.org",
    "codeberg.org",
    "git.sr.ht",
    "tangled.org",
    "git.gay",
    "gitee.com",
];
const GITHUB_DOMAINS: &[&str] = &["github.com"];
const GITLAB_DOMAINS: &[&str] = &["gitlab.com"];
const CURSEFORGE_DOMAINS: &[&str] = &["curseforge.com"];
const GOOGLE_FORMS_DOMAINS: &[&str] = &["docs.google.com"];
const GOOGLE_FORMS_SHORT_DOMAINS: &[&str] = &["forms.gle"];
const MICROSOFT_FORMS_DOMAINS: &[&str] = &[
    "forms.office.com",
    "forms.microsoft.com",
    "forms.cloud.microsoft",
];
const TYPEFORM_DOMAINS: &[&str] = &["typeform.com"];
const DISCORD_INVITE_DOMAINS: &[&str] = &["discord.com", "discordapp.com"];
const DISCORD_SHORT_INVITE_DOMAINS: &[&str] = &["discord.gg"];
const DISCORD_REDIRECT_DOMAINS: &[&str] = &["dsc.gg"];
const LICENSE_DOMAINS: &[&str] = &[
    "spdx.org",
    "opensource.org",
    "choosealicense.com",
    "gnu.org",
    "apache.org",
    "creativecommons.org",
];
const DONATION_DOMAINS: &[(&str, &[&str])] = &[
    ("patreon", &["patreon.com"]),
    (
        "bmac",
        &["buymeacoffee.com", "buymeacoff.ee", "coff.ee", "bmc.link"],
    ),
    (
        "paypal",
        &["paypal.me", "paypal.com", "py.pl", "paypal.biz"],
    ),
    ("ko-fi", &["ko-fi.com"]),
];
const SOURCE_REQUIRING_LICENSES: &[&str] = &[
    "GPL-2.0",
    "GPL-2.0+",
    "GPL-2.0-only",
    "GPL-2.0-or-later",
    "GPL-3.0",
    "GPL-3.0+",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "LGPL-2.1",
    "LGPL-2.1+",
    "LGPL-2.1-only",
    "LGPL-2.1-or-later",
    "LGPL-3.0",
    "LGPL-3.0+",
    "LGPL-3.0-only",
    "LGPL-3.0-or-later",
    "AGPL-3.0",
    "AGPL-3.0+",
    "AGPL-3.0-only",
    "AGPL-3.0-or-later",
    "MPL-2.0",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LinkTarget {
    pub field: String,
    pub url: String,
    pub image: bool,
}

impl LinkTarget {
    fn nag(&self, reason: &str, severity: ProjectNagSeverity) -> ProjectNag {
        ProjectNag::new(ProjectNagKind::LinkValidation, severity).with_details(
            json!({
                "field": self.field,
                "url": self.url,
                "reason": reason,
            }),
        )
    }

    fn required(&self, reason: &str) -> ProjectNag {
        self.nag(reason, ProjectNagSeverity::Required)
    }

    fn warning(&self, reason: &str) -> ProjectNag {
        self.nag(reason, ProjectNagSeverity::Warning)
    }
}

pub(super) fn targets(project: &Project) -> Vec<LinkTarget> {
    let mut targets = project
        .link_urls
        .iter()
        .map(|(field, link)| LinkTarget {
            field: field.clone(),
            url: link.url.clone(),
            image: false,
        })
        .collect::<Vec<_>>();
    if let Some(url) = &project.license.url {
        targets.push(LinkTarget {
            field: "license".into(),
            url: url.clone(),
            image: false,
        });
    }
    targets.sort_by(|a, b| a.field.cmp(&b.field));
    targets.extend(description::extract(&project.description));
    targets
}

pub(super) fn validate(
    project: &Project,
    versions: &[Version],
) -> Vec<ProjectNag> {
    let mut nags = validate_static(project);
    if project.link_urls.is_empty() {
        nags.push(ProjectNag::new(
            if project.components.minecraft_server.is_some() {
                ProjectNagKind::AddLinksServer
            } else {
                ProjectNagKind::AddLinks
            },
            ProjectNagSeverity::Suggestion,
        ));
    }
    let is_source_project = project
        .project_types
        .iter()
        .any(|kind| matches!(kind.as_str(), "mod" | "plugin"));
    let is_datapack =
        project.project_types.iter().any(|kind| kind == "datapack");
    let has_source = project
        .link_urls
        .get("source")
        .is_some_and(|link| !link.url.is_empty());
    if !is_datapack
        && is_source_project
        && SOURCE_REQUIRING_LICENSES.contains(&project.license.id.as_str())
        && !has_source
        && !versions.iter().all(|version| version.files.len() >= 2)
    {
        let (project_type, _) =
            LegacyProject::get_project_type(&project.project_types);
        nags.push(
            ProjectNag::new(
                ProjectNagKind::GplLicenseSourceRequired,
                ProjectNagSeverity::Required,
            )
            .with_details(
                json!({ "project_type": project_type, "field": "source" }),
            ),
        );
    }
    nags
}

pub(super) fn validate_static(project: &Project) -> Vec<ProjectNag> {
    let targets = targets(project);
    validate_targets_static(&targets)
}

pub(super) fn validate_targets_static(
    targets: &[LinkTarget],
) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    for target in targets {
        if let Some(nag) = validate_target(target) {
            nags.push(nag);
        }
    }
    let mut seen: HashMap<&str, Vec<&LinkTarget>> = HashMap::new();
    for target in targets
        .iter()
        .filter(|target| target.field != "description")
    {
        seen.entry(&target.url).or_default().push(target);
    }
    for group in seen.values().filter(|group| group.len() > 1) {
        for target in group {
            let Some(other) =
                group.iter().find(|other| other.field != target.field)
            else {
                continue;
            };
            let mut nag = target.required("duplicate");
            nag.details["other_field"] = json!(other.field);
            nags.push(nag);
        }
    }
    nags.sort_by(|a, b| a.details.to_string().cmp(&b.details.to_string()));
    nags.dedup();
    nags
}

pub(super) fn validate_target(target: &LinkTarget) -> Option<ProjectNag> {
    if target.url.len() > 2048 {
        return Some(target.required("malformed"));
    }
    let Ok(url) = Url::parse(&target.url) else {
        return Some(target.required("malformed"));
    };
    if target.field == "description" && url.scheme() == "file" {
        return Some(target.required("download"));
    }
    if !matches!(url.scheme(), "https" | "http")
        || (target.field != "description" && url.scheme() != "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Some(target.required("malformed"));
    }
    if globally_blocked(&url) {
        return Some(target.required("global_blocklist_match"));
    }
    if target.field == "description" {
        return description::known_download(&url, target.image)
            .then(|| target.required("download"));
    }
    if !matches!(url.host(), Some(url::Host::Domain(_))) {
        return Some(target.required("ip_address"));
    }
    if let Some(reason) = field_block(&target.field, &url) {
        return Some(target.required(reason));
    }
    if target.field == "source"
        && from_domains(&url, SOURCE_DOMAINS)
        && !repository_path(&url)
    {
        return Some(target.required("source_repository"));
    }
    if !matches!(
        target.field.as_str(),
        "site" | "store" | "other" | "source" | "discord" | "wiki"
    ) && !allowed(&target.field, &url)
    {
        return Some(target.warning("not_in_allowlist"));
    }
    None
}

pub(super) fn validate_input(
    links: &HashMap<String, String>,
    license_url: Option<&str>,
    description: &str,
) -> Vec<ProjectNag> {
    let mut targets = links
        .iter()
        .map(|(field, url)| LinkTarget {
            field: field.clone(),
            url: url.clone(),
            image: false,
        })
        .collect::<Vec<_>>();
    if let Some(url) = license_url {
        targets.push(LinkTarget {
            field: "license".into(),
            url: url.into(),
            image: false,
        });
    }
    targets.extend(self::description::extract(description));
    validate_targets_static(&targets)
}

pub(super) fn globally_blocked(url: &Url) -> bool {
    from_domains(url, GLOBAL_BLOCKS) || url.host_str().is_some_and(is_nsfw_host)
}

fn is_nsfw_host(host: &str) -> bool {
    let normalized = host.trim_end_matches('.').to_ascii_lowercase();
    let mut suffix = normalized.as_str();
    loop {
        if blocklist::is_porn(suffix) {
            return true;
        }
        let Some((_, rest)) = suffix.split_once('.') else {
            return false;
        };
        suffix = rest;
    }
}

fn host(url: &Url) -> &str {
    url.host_str().unwrap_or_default().trim_end_matches('.')
}

fn domain_matches(host: &str, domain: &str) -> bool {
    host.eq_ignore_ascii_case(domain)
        || host
            .to_ascii_lowercase()
            .strip_suffix(domain)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

fn from_domains(url: &Url, domains: &[&str]) -> bool {
    domains
        .iter()
        .any(|domain| domain_matches(host(url), domain))
}

fn path(url: &Url) -> Vec<&str> {
    url.path_segments()
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect()
}

fn repository_path(url: &Url) -> bool {
    let parts = path(url);
    parts.len() >= 2
        && !matches!(
            parts[0],
            "sponsors" | "settings" | "login" | "join" | "explore" | "topics"
        )
}

fn repo_section(url: &Url, section: &str) -> bool {
    let parts = path(url);
    repository_path(url)
        && (parts.get(2) == Some(&section)
            || (from_domains(url, GITLAB_DOMAINS)
                && parts.windows(2).any(|pair| pair == ["-", section])))
}

pub(super) fn discord_code(url: &Url) -> Option<&str> {
    let parts = path(url);
    let code = if from_domains(url, DISCORD_SHORT_INVITE_DOMAINS)
        && parts.len() == 1
    {
        parts[0]
    } else if from_domains(url, DISCORD_INVITE_DOMAINS)
        && parts.len() == 2
        && parts[0] == "invite"
    {
        parts[1]
    } else {
        return None;
    };
    (!code.is_empty()
        && code
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'))
    .then_some(code)
}

fn allowed(field: &str, url: &Url) -> bool {
    let parts = path(url);
    match field {
        "source" => {
            from_domains(url, SOURCE_DOMAINS)
                && repository_path(url)
                && !repo_section(url, "issues")
                && !repo_section(url, "wiki")
        }
        "issues" => {
            (from_domains(url, SOURCE_DOMAINS) && repo_section(url, "issues"))
                || (from_domains(url, CURSEFORGE_DOMAINS)
                    && parts.len() == 4
                    && parts[0] == "minecraft"
                    && parts[3] == "issues")
                || (from_domains(url, GOOGLE_FORMS_DOMAINS)
                    && parts.first() == Some(&"forms"))
                || (from_domains(url, GOOGLE_FORMS_SHORT_DOMAINS)
                    && !parts.is_empty())
                || (from_domains(url, MICROSOFT_FORMS_DOMAINS)
                    && !parts.is_empty())
                || (from_domains(url, TYPEFORM_DOMAINS)
                    && parts.first() == Some(&"to")
                    && parts.len() >= 2)
        }
        "wiki" => {
            from_domains(url, SOURCE_DOMAINS) && repo_section(url, "wiki")
        }
        "discord" => discord_code(url).is_some(),
        "github" => {
            from_domains(url, GITHUB_DOMAINS)
                && parts.first() == Some(&"sponsors")
                && (parts.len() == 2
                    || (parts.len() == 3 && parts[2] == "sponsorships"))
        }
        "license" => {
            from_domains(url, LICENSE_DOMAINS)
                || (from_domains(url, GITHUB_DOMAINS) && allowed("source", url))
        }
        _ => DONATION_DOMAINS.iter().any(|(platform, domains)| {
            *platform == field && from_domains(url, domains)
        }),
    }
}

fn field_block(field: &str, url: &Url) -> Option<&'static str> {
    if field == "wiki" && from_domains(url, SOURCE_DOMAINS) {
        return None;
    }
    let own_pattern = allowed(field, url);
    for other in [
        "issues", "wiki", "discord", "github", "patreon", "bmac", "paypal",
        "ko-fi", "license",
    ] {
        let matches_other = if other == "license" {
            from_domains(url, LICENSE_DOMAINS)
        } else {
            allowed(other, url)
        };
        if other != field && matches_other {
            return Some("wrong_field");
        }
    }
    if field != "source" && from_domains(url, SOURCE_DOMAINS) && !own_pattern {
        return Some("wrong_field");
    }
    if field != "discord" && from_domains(url, DISCORD_REDIRECT_DOMAINS) {
        return Some("wrong_field");
    }
    let explicit_exception = matches!(
        field,
        "source" | "issues" | "wiki" | "github" | "discord" | "license"
    ) && own_pattern;
    if from_domains(url, EXTERNAL_BLOCKS) && !explicit_exception {
        return Some("external_blocklist_match");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(field: &str, url: &str) -> Option<ProjectNag> {
        validate_target(&LinkTarget {
            field: field.into(),
            url: url.into(),
            image: false,
        })
    }

    #[test]
    fn global_blocklist_matches_apply_to_every_field_and_subdomain() {
        for field in [
            "source",
            "issues",
            "wiki",
            "license",
            "description",
            "patreon",
            "site",
        ] {
            assert_eq!(
                check(field, "https://www.example.com/").unwrap().details["reason"],
                "global_blocklist_match"
            );
        }
        assert!(!globally_blocked(
            &Url::parse("https://notexample.com").unwrap()
        ));
        assert!(globally_blocked(
            &Url::parse("https://BIT.LY./test").unwrap()
        ));
    }

    #[test]
    fn specific_routes_distinguish_shared_hosts() {
        for (field, url) in [
            ("issues", "https://github.com/modrinth/code/issues"),
            ("wiki", "https://github.com/modrinth/code/wiki"),
            ("wiki", "https://github.com/modrinth/code"),
            (
                "wiki",
                "https://github.com/modrinth/code/blob/main/README.md",
            ),
            ("wiki", "https://github.com/modrinth/code/issues"),
            (
                "wiki",
                "https://gitlab.com/group/repo/-/blob/main/README.md",
            ),
            ("source", "https://github.com/modrinth/code"),
            ("license", "https://github.com/modrinth/code"),
            (
                "license",
                "https://github.com/modrinth/code/blob/main/LICENSE",
            ),
            ("github", "https://github.com/sponsors/modrinth"),
            (
                "issues",
                "https://legacy.curseforge.com/minecraft/mc-mods/map/issues",
            ),
            ("issues", "https://gitlab.com/group/repo/-/issues"),
            ("discord", "https://discordapp.com/invite/modrinth"),
        ] {
            assert!(check(field, url).is_none(), "{field}: {url}");
        }
        for (field, url) in [
            ("license", "https://github.com/modrinth"),
            (
                "license",
                "https://gitlab.com/group/repo/-/blob/main/LICENSE",
            ),
            ("license", "https://github.com/modrinth/code/issues"),
            ("license", "https://github.com/modrinth/code/wiki"),
            ("source", "https://github.com/modrinth/code/issues"),
            ("site", "https://discord.gg/modrinth"),
            ("other", "https://ko-fi.com/modrinth"),
            ("issues", "https://www.google.com/search?q=forms"),
            ("discord", "https://discord.com/channels/1/2"),
        ] {
            assert_eq!(
                check(field, url).unwrap().severity,
                ProjectNagSeverity::Required,
                "{field}: {url}"
            );
        }
    }

    #[test]
    fn misplaced_links_take_precedence_over_external_blocklist_matches() {
        for (field, url, reason) in [
            ("site", "https://discord.gg/modrinth", "wrong_field"),
            (
                "source",
                "https://github.com/sponsors/modrinth",
                "wrong_field",
            ),
            (
                "source",
                "https://github.com/modrinth/code/issues",
                "wrong_field",
            ),
            (
                "site",
                "https://youtube.com/@modrinth",
                "external_blocklist_match",
            ),
            ("site", "https://bit.ly/project", "global_blocklist_match"),
        ] {
            let nag = check(field, url).unwrap();
            assert_eq!(nag.details["reason"], reason, "{field}: {url}");
            assert_eq!(nag.severity, ProjectNagSeverity::Required);
        }
    }

    #[test]
    fn ip_addresses_cannot_be_external_fields() {
        for url in ["https://127.0.0.1", "https://[::1]", "https://2130706433"]
        {
            assert_eq!(
                check("site", url).unwrap().details["reason"],
                "ip_address"
            );
        }
    }

    #[test]
    fn duplicates_compare_only_raw_strings_including_license() {
        let targets = [
            LinkTarget {
                field: "site".into(),
                url: "https://project.dev/docs".into(),
                image: false,
            },
            LinkTarget {
                field: "wiki".into(),
                url: "https://project.dev/docs/".into(),
                image: false,
            },
            LinkTarget {
                field: "license".into(),
                url: "https://project.dev/docs".into(),
                image: false,
            },
        ];
        let nags = validate_targets_static(&targets);
        let duplicates = nags
            .iter()
            .filter(|nag| nag.details["reason"] == "duplicate")
            .collect::<Vec<_>>();
        assert_eq!(duplicates.len(), 2);
        assert!(!duplicates.iter().any(|nag| nag.details["field"] == "wiki"));
    }

    #[test]
    fn repeated_fields_only_report_duplicates_with_other_fields() {
        let license = LinkTarget {
            field: "license".into(),
            url: "https://project.dev/license".into(),
            image: false,
        };
        let mut targets = vec![license.clone(), license];
        let nags = validate_targets_static(&targets);
        assert!(!nags.iter().any(|nag| nag.details["reason"] == "duplicate"));

        targets.push(LinkTarget {
            field: "site".into(),
            url: "https://project.dev/license".into(),
            image: false,
        });
        let nags = validate_targets_static(&targets);
        let duplicates = nags
            .iter()
            .filter(|nag| nag.details["reason"] == "duplicate")
            .collect::<Vec<_>>();
        assert_eq!(duplicates.len(), 2);
        assert!(duplicates.iter().any(|nag| {
            nag.details["field"] == "license"
                && nag.details["other_field"] == "site"
        }));
        assert!(duplicates.iter().any(|nag| {
            nag.details["field"] == "site"
                && nag.details["other_field"] == "license"
        }));
    }

    #[test]
    fn donation_aliases_and_unknown_hosts() {
        for (field, url) in [
            ("paypal", "https://www.paypal.me/violamcmod"),
            ("bmac", "https://bmc.link/creator"),
            ("bmac", "https://coff.ee/creator"),
            ("paypal", "https://py.pl/payment"),
        ] {
            assert!(check(field, url).is_none());
        }
        assert!(check("wiki", "https://docs.myproject.dev").is_none());
        assert!(
            check("source", "https://git.myproject.dev/owner/repo").is_none()
        );
    }
}
