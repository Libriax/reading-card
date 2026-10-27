# Security

Please report vulnerabilities privately to **admin@libriax.com**, not in a public issue.

Include what you found, how to reproduce it, and what it affects. You'll get a reply within a few days.

## Scope

- This Worker: anything that lets a card run code, load outside content, or make the Worker fetch somewhere it shouldn't
- The Libriax card endpoint (`/api/card/<username>`): anything that exposes reading data for a user who hasn't turned their card on
