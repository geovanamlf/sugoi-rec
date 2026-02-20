# TODO: Implement in-memory cache for AniList anime metadata.
# The idea is to store anime data locally so we don't have to call
# AniList every time. Each entry will have a TTL (time to live) so
# outdated data gets refreshed automatically.