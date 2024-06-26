/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import {Banner, BannerKind} from '../Banner';
import {Internal} from '../Internal';
import {Tooltip} from '../Tooltip';
import {Divider} from '../components/Divider';
import GatedComponent from '../components/GatedComponent';
import {T} from '../i18n';
import {
  MAX_FILES_ALLOWED_FOR_DIFF_STAT,
  SLOC_THRESHOLD_FOR_SPLIT_SUGGESTIONS,
} from '../sloc/diffStatConstants';
import {useFetchSignificantLinesOfCode} from '../sloc/useFetchSignificantLinesOfCode';
import {SplitButton} from '../stackEdit/ui/SplitButton';
import {type CommitInfo} from '../types';
import {Icon} from 'shared/Icon';

function SplitSuggestionImpl({commit}: {commit: CommitInfo}) {
  const significantLinesOfCode = useFetchSignificantLinesOfCode(commit);
  if (
    significantLinesOfCode == null ||
    significantLinesOfCode <= SLOC_THRESHOLD_FOR_SPLIT_SUGGESTIONS
  ) {
    return null;
  }
  return (
    <>
      <Divider />
      <Banner
        tooltip=""
        kind={BannerKind.green}
        icon={<Icon icon="info" />}
        alwaysShowButtons
        buttons={
          <SplitButton
            style={{
              border: '1px solid var(--button-secondary-hover-background)',
            }}
            trackerEventName="SplitOpenFromSplitSuggestion"
            commit={commit}
          />
        }>
        <div>
          <T>Pro tip: Small Diffs lead to less SEVs, quicker review times and happier teams.</T>
          &nbsp;
          <Tooltip
            inline={true}
            trigger="hover"
            title={`Significant Lines of Code (SLOC): ${significantLinesOfCode}, this puts your diff in the top 10% of diffs. `}>
            <T>This diff is a bit big</T>
          </Tooltip>
          <T>, consider splitting it up</T>
        </div>
      </Banner>
    </>
  );
}

export default function SplitSuggestion({commit}: {commit: CommitInfo}) {
  if (commit.totalFileCount > MAX_FILES_ALLOWED_FOR_DIFF_STAT) {
    return null;
  }
  // using a gated component here to avoid exposing when diff size is too big  to show the split suggestion
  return (
    <GatedComponent featureFlag={Internal.featureFlags?.ShowSplitSuggestion}>
      <SplitSuggestionImpl commit={commit} />
    </GatedComponent>
  );
}
