# TODO: Implement rate limit control for AniList API calls.
# AniList currently allows up to 30 requests per minute (degraded state).
# The official limit is 90 req/min but has been temporarily reduced.
# This module will track how many requests have been made and block calls
# that would exceed the limit, avoiding API bans.
# The response headers X-RateLimit-Remaining and Retry-After will be used
# to control this dynamically instead of relying on a fixed counter.