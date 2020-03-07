- Feature Name: `library`
- Start Date: 2020-02-07

# Summary
[summary]: #summary

In addition to the command line interface, `ctftool` provides a Rust API.

# Motivation
[motivation]: #motivation

Testability. At this moment, having a library makes it easier to write unit tests, as well as to debug integration
tests by making all the code run in a single process.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

`ctftool` crate provides a library and a binary. All the binary does is calling the `main` function from the library.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

While library implements the command-line interface, it tries to decouple itself from the environment - command-line
arguments and working directory must be passed to it.

# Drawbacks
[drawbacks]: #drawbacks

None.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

In order to improve debugging experience, instead of forcing all the code into a single process, one could have improved
logging and error reporting. However, author is of the opinion that they are supposed to complement rather than replace
single-stepping.

# Prior art
[prior-art]: #prior-art

JGit.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

Should there be a public API?

How to handle I/O? Right now the library interacts with stdin/stdout/stderr, which is bad. It also executes `git`, which
does the same.

# Future possibilities
[future-possibilities]: #future-possibilities

Public API.

I/O wrapping - possibly using `AsyncRead` and `AsyncWrite` and custom pipes for `std::process`.
