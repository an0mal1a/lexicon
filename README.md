# Lexicon

Lexicon is the daily semantic word game built on the published `lxdb` crate.
LXDB remains a reusable, game-agnostic dependency: this repository owns game
rules, HTTP persistence and the Nuxt client.

Before publishing LXDB, the game uses its sibling checkout through a Cargo
`path` dependency. Once `lxdb` is on crates.io, remove that path and retain
the released version constraint.
