# TODO: Implement rate limit control for AniList API calls.
# AniList allows up to 90 requests per minute. This module will
# track how many requests have been made and block calls that
# would exceed the limit, avoiding API bans.