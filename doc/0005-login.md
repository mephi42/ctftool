- Feature Name: `login`
- Start Date: 2020-01-02

# Summary
[summary]: #summary

`ctf login` command logs into CTF website and saves the cookies.

# Motivation
[motivation]: #motivation

Some CTFs provide tasks only to logged in users. Therefore, it's essential to support authentication in `ctftool`.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

For CTF websites requiring authentication, user needs to invoke `ctf login` before invoke `ctf fetch`. `ctf login` will
ask for login and password, perform engine detection if necessary, perform the login procedure, and persist the results.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

`ctf login` persists cookies in the `.ctfcredentials` file. This file is not tracked in git.

# Drawbacks
[drawbacks]: #drawbacks

If cookies stored on disk are stolen, they may be used to impersonate the respective CTF website user.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

Credentials themselves could be stored in `.ctfcredentials` instead of cookies. However, credentials are more sensitive
than cookies; also, storing cookies avoids having to authenticate on each `ctf fetch`.

# Prior art
[prior-art]: #prior-art

`docker login` command. `gitcredentials`.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

None.

# Future possibilities
[future-possibilities]: #future-possibilities

Make `ctf fetch` invoke `ctf login` when necessary instead of failing.

Add operating system keyring integration as a `.ctfcredentials` alternative.
