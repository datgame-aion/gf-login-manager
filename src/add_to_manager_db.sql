INSERT INTO cookies (
        display_name,
        creation_utc,
        value,
        expires_utc,
        last_access_utc
    )
VALUES (
        :display_name,
        :creation_utc,
        :value,
        :expires_utc,
        :last_access_utc
    )