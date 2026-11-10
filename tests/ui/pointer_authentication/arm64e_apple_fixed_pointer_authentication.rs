// ignore-tidy-linelength
// The arm64e ABI fixes pointer authentication on Apple targets, so `-Z pointer-authentication` is
// an error there rather than an option that is quietly ignored.
//@ ignore-backends: gcc
//@ check-fail
//@ needs-llvm-components: aarch64
//@ compile-flags: -Zpointer-authentication=-calls --crate-type=lib --target arm64e-apple-darwin

#![feature(no_core)]
#![no_std]
#![no_main]
#![no_core]

//~? ERROR `-Z pointer-authentication` cannot change pointer authentication for target arm64e-apple-darwin, whose ABI fixes it
