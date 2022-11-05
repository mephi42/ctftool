- Feature Name: `challenge`
- Start Date: 2022-11-05

# Summary
[summary]: #summary

`ctftool` can be used to manually add challenges.

# Motivation
[motivation]: #motivation

If a CTF website is not supported by `ctftool`, or if the support is buggy, challenges need to be downloaded and added
manually; bogus challenges need to be removed or edited manually.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

In order to see a list of challenges, do `ctf challenge show`.

In order to add a challenge, do `ctf challenge add chal`.

In order to edit the challenge description, do `ctf challenge set-description chal "zajebiste pwn"`.

In order to remove a challenge, do `ctf challenge rm chal`.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

`challenge` subcommands (`show`, `add`, `set-description`, `rm`) work exclusively with the `.ctf` file. In particular,
they do not download, create, overwrite or delete anything.

`ctf challenge show` prints a list of challenges and their descriptions.

`ctf challenge add NAME` adds a new challenge with the name `NAME`. The subdirectory called `NAME` must exist.

`ctf challenge set-description NAME DESCRIPTION` set the description of challenge `NAME` to `DESCRIPTION`.

`ctf challenge rm NAME` removes an existing challenge with the name `NAME`.

# Drawbacks
[drawbacks]: #drawbacks

None.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

Alternative 1: it should be very easy to add or adjust support for CTF websites.

Alternative 2: a remote engine for local directories should be added. Users can then manually put challenges into such
directories and `ctf fetch` them.

# Prior art
[prior-art]: #prior-art

`git remote` command.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

None.

# Future possibilities
[future-possibilities]: #future-possibilities

None.
