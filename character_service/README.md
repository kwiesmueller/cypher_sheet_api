# Cypher Sheet Character Service

This API is a first attempt at syncing characters from the
[cypher_sheet](https://github.com/kwiesmueller/cypher_sheet) app with
a cloud service.

The goal is to provide users a safe access to their characters behind social
logins.

This API is also required for a web version of the app.

## Authentication (not implemented yet)

The plan is to use the free tier of Firebase Auth to offer various social
logins.
The user would sign-in from the app and send their signed token with every
request to the API. The server would then validate the authenticity of the
token and allow the user access to their characters.

## Storage (work in progress)

In its current state storage is entirely file based which comes with some
limitations.

The user will request a new character which results in the API allocating a
UUID for it and returning it to the user.
The user can then send new revisions for that UUID.

Any revisions are currently stored as proto files in a directory per character.
This matches what is done in the app right now.

The intention behind this (somewhat odd) storage design is to avoid any
dependencies on database or storage services. But as a result it will not scale
to the intended amount of users and characters.

To support more load and satisfy reliability, data integrity, and cost
expectations a few options have been considered:

* A cloud storage adapter to store data in an S3 compliant API
  ([minio](https://min.io/),
  [Cloudflare R2](https://developers.cloudflare.com/r2/))
* A database adapter (PostgreSQL, MongoDB, etc.)
* Switching to Firebase

Right now the decision leans towards S3 (Cloudflare R2) as it seems like the
most cost effective path while still enabling growth.
Minio would be a nice alternative as it can be operated on the same Kubernetes
Cluster, but intially I would prefer outsourcing the responsibility of storage.

The same applies to running a database. And hosted solutions are not as cost
effective for an unfinanced fun project.

Firebase while generally a great start for projects like this, sadly has a quite
strong vendor lock-in and steep cost curve. Regulating cost and load seems more
straight forward with a self-managed API server talking to e.g. R2.