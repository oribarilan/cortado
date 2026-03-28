# Optional: Panel Filter + Fuzzy Finding

Add a filter/search bar to the panel for narrowing activities when the list grows large.

## Details

- Command-palette style: list focused on open, typing redirects to filter input
- Match against: activity title, feed name, status field values
- Multi-token matching: "github failing" matches if all tokens appear in the combined searchable text
- Fuzzy matching (fzf-style) as a stretch — start with case-insensitive substring per token
- Filter resets on panel hide
- Activity count badge updates to reflect filtered count
- Esc clears filter (if non-empty) or closes panel (if empty)

## Why deferred

With 5-20 activities and the priority section surfacing attention items at the top, ↑↓ navigation is fast enough. Filter becomes valuable if activity count grows significantly.
