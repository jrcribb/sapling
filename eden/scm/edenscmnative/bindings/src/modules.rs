/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use cpython::PyModule;
use cpython::PyResult;
use cpython::Python;
use cpython_ext::PyNone;
use paste::paste;

macro_rules! add_modules {
    ( $py:ident, $m:ident, [ $( $name:ident, )* ] ) => {
        let name = $m.get($py, "__name__")?.extract::<String>($py)?;
        $(
            paste! {
                $m.add($py, stringify!($name), ::[<py $name>]::init_module($py, &name)?)?;
            }
         )*
    }
}

/// Populate an existing empty module so it contains utilities.
pub(crate) fn populate_module(py: Python<'_>, module: &PyModule) -> PyResult<PyNone> {
    let m = module;
    m.add(py, "__doc__", "Mercurial Rust Bindings")?;
    add_modules!(
        py,
        m,
        [
            auth,
            blackbox,
            bytes,
            cats,
            checkout,
            clientinfo,
            cliparser,
            configparser,
            dag,
            diffhelpers,
            dirs,
            doctor,
            drawdag,
            eagerepo,
            edenapi,
            error,
            exchange,
            fail,
            fs,
            gitstore,
            hgmetrics,
            hgtime,
            io,
            lock,
            lz4,
            manifest,
            metalog,
            mutationstore,
            nodemap,
            pathhistory,
            pathmatcher,
            pprint,
            process,
            progress,
            refencode,
            regex,
            renderdag,
            repo,
            revisionstore,
            revlogindex,
            sptui,
            status,
            threading,
            tracing,
            treestate,
            vlq,
            worker,
            workingcopy,
            zstd,
            zstore,
        ]
    );

    Ok(PyNone)
}
