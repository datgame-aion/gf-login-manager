SELECT creation_utc,
    value,
    expires_utc,
    last_access_utc
    FROM cookies
where name = 'gf-token-production'