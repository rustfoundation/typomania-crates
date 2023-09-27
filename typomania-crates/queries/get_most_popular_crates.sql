SELECT
    crates.name AS name,
    COALESCE(users.gh_login, teams.login) AS login,
    crates.homepage AS homepage,
    crates.repository AS repository,
    crates.documentation AS documentation,
    crates.description AS description,
    crates.downloads AS downloads
FROM (
    SELECT crates.*, recent_crate_downloads.downloads AS recent_downloads
    FROM crates
    LEFT JOIN recent_crate_downloads ON (crates.id = recent_crate_downloads.crate_id)
    WHERE recent_crate_downloads.downloads IS NOT NULL
    ORDER BY recent_crate_downloads.downloads DESC
    LIMIT $1
) AS crates
LEFT JOIN crate_owners ON (crates.id = crate_owners.crate_id)
LEFT JOIN users ON (crate_owners.owner_id = users.id AND crate_owners.owner_kind = 0 AND NOT crate_owners.deleted)
LEFT JOIN teams ON (crate_owners.owner_id = teams.id AND crate_owners.owner_kind = 1 AND NOT crate_owners.deleted)
ORDER BY crates.recent_downloads DESC;
