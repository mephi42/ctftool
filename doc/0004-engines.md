- Feature Name: `engines`
- Start Date: 2020-01-01

# Summary
[summary]: #summary

`ctftool` supports different CTF engines. The user may explicitly associate an engine with a remote, otherwise `ctftool`
tries to detect it automatically.

# Motivation
[motivation]: #motivation

While a lot of CTFs use [CTFd](https://github.com/CTFd/CTFd), some use custom engines. It's important to be able to
quickly add support for such custom engines.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

`ctf remote get-engine <name>` shows an engine associated with a remote. By default, `auto` is used.

`ctf remote set-engine <name> <newengine>` associates an engine with a remote.

`ctf fetch <name>` uses an engine associated with a remote in order to download CTF metadata. `auto` engine tries all
supported engines sequentially, and chooses the one which produces the best results. The chosen engine is associated
with the remote.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

Engines are implemented by providing implementations of `Engine` trait and adding them to `ENGINES` hash map.

Automatic detection works by running all engines, throwing away all the failing ones and picking the one that fetched
the most challenges. In case of a draw, and arbitrary one is chosen. It's expected that the results are like `[Err, Err,
Ok(0 challenges), Ok(0 challenges), Ok(15 challenges)]`.

# Drawbacks
[drawbacks]: #drawbacks

Strictly speaking, automatic detection is not reliable, as two engines can be implemented, such that one can be confused
with another. However, it's not expected to happen in practice.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

`auto` could try different engines in parallel, but e.g. CTFd does not like it.

# Prior art
[prior-art]: #prior-art

None.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

How to support authentication? Many CTFd setups provide challenges only to logged in users.

# Future possibilities
[future-possibilities]: #future-possibilities

More engines!
