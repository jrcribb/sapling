/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import type {PrimitiveAtom} from 'jotai';
import type {AtomEffect, RecoilState} from 'recoil';

import {globalRecoil} from './AccessGlobalRecoil';
import serverAPI from './ClientToServerAPI';
import {getDefaultStore as globalJotai} from 'jotai';
import {atom as RecoilAtom} from 'recoil';

/**
 * Atom effect that clears the atom's value when the current working directory / repository changes.
 */
export function clearOnCwdChange<T>(): AtomEffect<T> {
  return ({resetSelf}) => serverAPI.onCwdChanged(resetSelf);
}

const jotaiStore = globalJotai();

/**
 * Creates a Recoil atom that is "entangled" with the Jotai atom.
 * Changing one atom automatically updates the other.
 */
export function entangledAtom<T>(
  jotaiAtom: PrimitiveAtom<T>,
  key: string,
  recoilEffects?: AtomEffect<T>[],
): RecoilState<T> {
  const initialValue = jotaiStore.get(jotaiAtom);

  let recoilValue = initialValue;
  let jotaiValue = initialValue;

  jotaiAtom.debugLabel = key;
  jotaiStore.sub(jotaiAtom, () => {
    const value = (jotaiValue = jotaiStore.get(jotaiAtom));
    if (recoilValue !== value) {
      // Recoil value is outdated.
      globalRecoil().set(recoilAtom, value);
    }
  });

  const effects = recoilEffects ?? [];
  effects.push(({onSet}) => {
    onSet(newValue => {
      if (jotaiValue !== newValue) {
        // Jotai value is outdated.
        recoilValue = newValue;
        jotaiStore.set(jotaiAtom, newValue);
      }
    });
  });

  const recoilAtom = RecoilAtom<T>({
    key,
    default: initialValue,
    effects,
  });

  return recoilAtom;
}
