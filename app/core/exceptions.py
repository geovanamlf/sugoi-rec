class AniListRateLimitError(Exception):
    """Raised when the AniList API rate limit is exceeded (HTTP 429)."""
    pass


class AniListUnavailableError(Exception):
    """Raised when the AniList API returns an unexpected error."""
    pass


class AnimeNotFoundError(Exception):
    """Raised when an anime doesn't exist on AniList."""
    pass