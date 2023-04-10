- Feature Name: `service`
- Start Date: 2023-04-12

# Summary
[summary]: #summary

`ctftool` can be used to manually add services.

# Motivation
[motivation]: #motivation

If a CTF website is not supported by `ctftool`, or if the support is buggy, services need to be added manually; bogus
services need to be removed or edited manually.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

In order to see a list of services, do `ctf service show` in a challenge directory.

In order to add a service, do `ctf service add service-name 1.2.3.4:5678` in a challenge directory.

In order to edit the service URL, do `ctf service set-url service-name 9.10.11.12:1314` in a challenge directory.

In order to remove a service, do `ctf service rm service-name` in a challenge directory.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

`service` subcommands (`show`, `add`, `set-url`, `rm`) work exclusively with the `.ctf` file. They must be run in a
challenge directory.

`ctf service show` prints a list of services and their URLs.

`ctf service add NAME URL` adds a new service with the name `NAME` and URL `URL`.

`ctf service set-url NAME URL` set the URL of the service `NAME` to `URL`.

`ctf service rm NAME` removes an existing service with the name `NAME`.

# Drawbacks
[drawbacks]: #drawbacks

None.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

Alternative 1: it should be very easy to add or adjust support for CTF websites.

Alternative 2: a remote engine for local directories should be added. Users can then manually put service descriptions
into such directories (e.g. into `DIR/CHALLENGE/services/SERVICE`) and `ctf fetch` them.

# Prior art
[prior-art]: #prior-art

`git remote` command.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

None.

# Future possibilities
[future-possibilities]: #future-possibilities

None.
