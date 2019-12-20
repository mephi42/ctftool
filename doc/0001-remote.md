- Feature Name: `remote`
- Start Date: 2019-12-20

# Summary
[summary]: #summary

`ctf remote` manages links between the local directory and the CTF websites.

# Motivation
[motivation]: #motivation

In order to download challenges, one needs to know on which website they are hosted.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

In order to see active links, do `ctf remote show`.

In order to link to CTF website, do `ctf remote add origin https://ctf.watevr.xyz`.

In order to remove the link, do `ctf remote rm origin`.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

`remote` subcommands (`show`, `add`, `rm`) work exclusively with `.ctf` file. In particular, they don't download
anything.

`ctf remote show` prints a list of configured link names.

`ctf remote add NAME URL` adds a new link with the name `NAME` that points to `URL`.

`ctf remote rm NAME` removes an existing link with the name `NAME`.

# Drawbacks
[drawbacks]: #drawbacks

None.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

`ctf remote` command allows to manage multiple remotes. It could manage just one, but extra flexibility is cheap in this
case.

# Prior art
[prior-art]: #prior-art

`git remote` command.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

How to make `ctf remote` an alias for `ctf remote show`? Clap does not seem to support that yet.

# Future possibilities
[future-possibilities]: #future-possibilities

`ctf clone URL` will call `ctf remote add origin URL`.
