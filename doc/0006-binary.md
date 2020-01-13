- Feature Name: `binary`
- Start Date: `2020-01-13`

# Summary
[summary]: #summary

`ctf binary` manages different versions of challenge binaries.

# Motivation
[motivation]: #motivation

In Jeopardy CTFs, it might be useful to have an original binary, a binary with debuginfo (e.g. from [dwarfexport](
https://github.com/ALSchwalm/dwarfexport)) and a patched binary (e.g. with antidebugging removed). In addition, in AD
CTFs one might have their own and other teams' hardened binaries. It's useful to have shortcuts to analyze, run,
compare, patch, exploit and deploy them.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

Binaries registered in `ctftool` are stored in git under `binary.alternative` names, e.g. `blackjack.hxp-v1` or
`libc.so.6.orig`. `ctf binary add` registers a new binary, which should follow this naming convention. `ctf binary rm`
removes a registered binary. `ctf binary default` makes a registered binary a default one, creating a copy named
`binary` (without `.alternative` part). `ctf binary show` displays registered binaries.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

`ctf binary` can be issued only from challenge directory, and operates on a respective challenge.

Which binary alternative is currently a default one is tracked in `.ctf` file. There might be no default alternative.

Registering an already registered binary and selecting an alternative which is already a default one are no-ops.

Removing a binary alternative removes not only its metadata from `.ctf`, but also its file on disk. Removing a default
binary alternative is allowed, in which case a default will be absent.

# Drawbacks
[drawbacks]: #drawbacks

None.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

Automatically registered binaries (e.g. by `checkout`) by default have no default alternatives and are thus not copied,
since they might be huge (e.g. VM images or challenge archives).

# Prior art
[prior-art]: #prior-art

`git add`, `git rm` and `cargo toolchain default`.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

None.

# Future possibilities
[future-possibilities]: #future-possibilities

Shortcuts for analyzing, running, comparing, patching, exploiting and deploying registered binaries.
