# share redirect contract

`/u/{owner}/{slug}` is a human-friendly alias only.

- It returns HTTP 302.
- It redirects to `/api/v1/publications/{publication_id}`.
- It does not replace publication ID as canonical identity.
- It never points at mutable draft/latest project state.

