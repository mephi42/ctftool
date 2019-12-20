- Feature Name: `fetch`
- Start Date: 2019-12-20

# Summary
[summary]: #summary

`ctf fetch` downloads challenge metadata from a CTF website.

# Motivation
[motivation]: #motivation

In order to download challenges, one needs to know at which exact URLs they are hosted.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

Use `ctf fetch` to initially download challenge metadata or to update it (e.g. when new challenges are opened or
existing ones are fixed).

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

`ctf fetch` works exclusively with `.ctf` file. It downloads only metadata, not binaries.

`ctf fetch` supports multiple different CTF website engines, and can automatically detect which one is used.

# Drawbacks
[drawbacks]: #drawbacks

None.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

`ctf fetch` could also download challenge binaries, but since they could be huge, and there is no way to know about this
in advance, this job will be assigned to `ctf checkout`. This is similar to `git lfs` workflow: `git fetch` downloads
only links to large files, and `git checkout` downloads them when necessary.

# Prior art
[prior-art]: #prior-art

`git fetch` command.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

How should credentials be stored in order to support cases when challenges are accessible only to logged in users?

# Future possibilities
[future-possibilities]: #future-possibilities

`ctf clone` will call `ctf fetch origin`.
