- Feature Name: `docker`
- Start Date: 2022-11-05

# Summary
[summary]: #summary

`ctftool` can be used to run challenges in Docker containers.

# Motivation
[motivation]: #motivation

Binary exploitation challenges often rely on a particular glibc or other system libraries. It is convenient to install
and debug them in a Docker image.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

In order to generate a Dockerfile, do `ctf docker init` in the challenge directory.

In order to run a command in a container, do `ctf docker exec -- bash`.

In order to delete the container, do `ctf docker rm`.

In order to delete the image, do `ctf docker rmi`.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

There is one Docker image and one Docker container per challenge. Sophisticated challenges may require more images or
containers, but automating that is futile.

`ctf docker init` generates `image/Dockerfile` and `docker-compose.yml`. It analyzes the challenge binaries and chooses
the best matching base image (`ubuntu:latest` by default) and system package versions. In addition, it adds the
following:

* gnome-terminal
* pwntools
* gdb, gdbserver
* gef
* glibc debuginfo and source code

`docker-compose.yml` contains various configuration bits (e.g., X11 forwarding).

`ctf docker exec COMMAND [ARGS ...]` builds the image and starts the container, and then execs the specified command
inside it.

`ctf docker rm` removes the container.

`ctf docker rmi` removes the image.

# Drawbacks
[drawbacks]: #drawbacks

`docker-compose` dependency.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

Alternative: use `LD_PRELOAD=libc.so` and manual `ld.so` invocation. The downside is that it's not easy to provide
glibc debug information.

# Prior art
[prior-art]: #prior-art

* https://github.com/io12/pwninit
* https://github.com/niklasb/libc-database

# Unresolved questions
[unresolved-questions]: #unresolved-questions

None.

# Future possibilities
[future-possibilities]: #future-possibilities

None.
