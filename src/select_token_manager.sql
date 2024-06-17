select display_name,
    creation_utc,
    value,
    expires_utc,
    last_access_utc
from cookies
where display_name = (:display_name)