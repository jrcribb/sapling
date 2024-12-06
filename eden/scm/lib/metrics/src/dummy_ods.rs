/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub(crate) type Counter = ();

pub(crate) fn new_counter(_name: &'static str) -> Counter {
    ()
}

pub(crate) fn increment(_counter: &Counter, _value: i64) {}
