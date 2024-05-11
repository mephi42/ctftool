- Feature Name: `url-rewrite`
- Start Date: 2020-02-07

# Summary
[summary]: #summary

`ctftool` can be configured to rewrite URLs before fetching them.

# Motivation
[motivation]: #motivation

Testability. Challenges may contain, e.g., Google Drive URLs, which may be handled by the code in a special way, but
which should not be fetched during testing. By rewriting these URLs to point to a test HTTP server, this special
handling can be verified without accessing external sites.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

Each remote is associated with a list of substitution regexes, which are all applied to URLs just before they are
fetched. There is no command line interface to control this feature, it is on `.ctf` file format level only.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

Each remote in `.ctf` file contains a `rewrite_rules` list, each element of which has `regex` and `rep` fields. All
elements are applied in order to each fetched URL using `Regex::replace_all`.

# Drawbacks
[drawbacks]: #drawbacks

This feature is not useful outside of testing.

Code becomes more complicated, since it's required to not forget to apply rewriting before each fetch.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

Testability can also be improved by mocking `reqwest::Client`. However, at the moment this is [too fragile](
https://github.com/seanmonstar/reqwest/issues/154).

An alternative to URL rewriting is proxying. However, for HTTPs URLs this would require a MITM proxy, which is
complicated to set up.

Finally, test data can be changed to point to a test HTTP server. However, the resulting URLs may be no longer matched
by the code, causing special handling not to occur at all.

# Prior art
[prior-art]: #prior-art

[mod_rewrite](https://httpd.apache.org/docs/current/mod/mod_rewrite.html).

# Unresolved questions
[unresolved-questions]: #unresolved-questions

None.

# Future possibilities
[future-possibilities]: #future-possibilities

None.
