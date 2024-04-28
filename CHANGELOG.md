# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- A separate feature `util` has been removed, now this functionality is always available.

- Added a new module `body_reader` in the [`tower-http-client`] to simplify the
  reading the response body in the most common cases.

- `tower_http_client::util::HttpClientExt` has been replaced by the
  `tower_http_client::ServiceExt`.

## [0.2.0] - 2024.04.21

The `tower-reqwest` has been splitted into two parts: [`tower-reqwest`] itself
with adapters for `tower-http` and [`tower-http-client`] with the useful utils
and extensions for creating an clients.

[`tower-http-client`]: tower-reqwest
[`tower-reqwest`]: tower-http-client
