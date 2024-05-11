- Feature Name: `checkout`
- Start Date: 2019-12-21

# Summary
[summary]: #summary

`ctf checkout` downloads CTF challenge binaries.

# Motivation
[motivation]: #motivation

Downloading all challenge binaries with just one command saves time initially, and also when new challenges are opened.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

When issued from the repository, `ctf checkout` downloads binaries for all challenges and `ctf checkout CHALLENGE`
downloads binaries for a particular challenge. Likewise, when issued from challenge directory, `ctf checkout` downloads
binaries for the respective challenge.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

First, `ctf checkout` determines the current working directory relative to the repository root, and from that infers
which challenges and binaries it should handle.

If some of the binaries obtained this way already exist on disk, it compares their checksums with those stored in `.ctf`
file (which originate from earlier invocations of `ctf fetch` or `ctf checkout`) and skips those binaries for which they
match.

The remaining binaries are downloaded in parallel.

Finally, `ctf checkout`, depending on whether they existed in the first place, verifies or updates checksums of
downloaded binaries.

Checksum and download activities for each binary are tracked with a progress bar.

# Drawbacks
[drawbacks]: #drawbacks

None.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

`ctf checkout` could also detect whether downloaded binaries are packed, and unpack them (with varying levels of
sophistication: `.gz`, `tar.gz`, `extract-vmlinux`, etc). However, the goal of `ctf checkout` is to let the user get the
initial impression of the challenges (e.g., by doing `tar -t`, `binwalk`, etc., manually), so indiscriminate unpacking
might be unnecessary. Therefore, it will be implemented separately.

`ctf checkout` could also be taught to download single binaries, however, this is too fine-grained.

# Prior art
[prior-art]: #prior-art

`git checkout` command.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

None.

# Future possibilities
[future-possibilities]: #future-possibilities

`ctf clone` will call `ctf checkout`.
