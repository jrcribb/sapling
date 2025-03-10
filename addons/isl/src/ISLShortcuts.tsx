/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import {Kbd} from 'isl-components/Kbd';
import {KeyCode, makeCommandDispatcher, Modifier} from 'isl-components/KeyboardShortcuts';
import {isMac} from 'isl-components/OperatingSystem';
import {useMemo} from 'react';
import {TypedEventEmitter} from 'shared/TypedEventEmitter';
import {t} from './i18n';
import {useModal} from './useModal';

import './ISLShortcuts.css';

const CMD = isMac ? Modifier.CMD : Modifier.CTRL;

/* eslint-disable no-bitwise */
export const [ISLCommandContext, useCommand, dispatchCommand, allCommands] = makeCommandDispatcher({
  OpenShortcutHelp: [Modifier.SHIFT, KeyCode.QuestionMark],
  ToggleSidebar: [CMD, KeyCode.Period],
  OpenUncommittedChangesComparisonView: [CMD, KeyCode.SingleQuote],
  OpenHeadChangesComparisonView: [[CMD, Modifier.SHIFT], KeyCode.SingleQuote],
  Escape: [Modifier.NONE, KeyCode.Escape],
  SelectUpwards: [Modifier.NONE, KeyCode.UpArrow],
  SelectDownwards: [Modifier.NONE, KeyCode.DownArrow],
  OpenDetails: [Modifier.NONE, KeyCode.RightArrow],
  ContinueSelectionUpwards: [Modifier.SHIFT, KeyCode.UpArrow],
  ContinueSelectionDownwards: [Modifier.SHIFT, KeyCode.DownArrow],
  SelectAllCommits: [Modifier.ALT, KeyCode.A],
  HideSelectedCommits: [Modifier.NONE, KeyCode.Backspace],
  ZoomIn: [Modifier.ALT, KeyCode.Plus],
  ZoomOut: [Modifier.ALT, KeyCode.Minus],
  ToggleTheme: [Modifier.ALT, KeyCode.T],
  ToggleShelvedChangesDropdown: [Modifier.ALT, KeyCode.S],
  ToggleDownloadCommitsDropdown: [Modifier.ALT, KeyCode.D],
  ToggleCwdDropdown: [Modifier.ALT, KeyCode.C],
  ToggleBulkActionsDropdown: [Modifier.ALT, KeyCode.B],
  ToggleFocusMode: [Modifier.ALT, KeyCode.F],
  ToggleBookmarksManagerDropdown: [Modifier.ALT, KeyCode.M],
  RebaseOntoCurrentStackBase: [Modifier.ALT, KeyCode.R],
});

export type ISLCommandName = Parameters<typeof useCommand>[0];

/** Like useCommand, but returns an eventEmitter you can subscribe to */
export function useCommandEvent(commandName: ISLCommandName): TypedEventEmitter<'change', null> {
  const emitter = useMemo(() => new TypedEventEmitter<'change', null>(), []);
  useCommand(commandName, () => {
    emitter.emit('change', null);
  });
  return emitter;
}

export const ISLShortcutLabels: Partial<Record<ISLCommandName, string>> = {
  Escape: t('Dismiss Tooltips and Popups'),
  OpenShortcutHelp: t('Open Shortcut Help'),
  ToggleSidebar: t('Toggle Commit Info Sidebar'),
  OpenUncommittedChangesComparisonView: t('Open Uncommitted Changes Comparison View'),
  OpenHeadChangesComparisonView: t('Open Head Changes Comparison View'),
  SelectAllCommits: t('Select All Commits'),
  ToggleTheme: t('Toggle Light/Dark Theme'),
  ZoomIn: t('Zoom In'),
  ZoomOut: t('Zoom Out'),
  ToggleShelvedChangesDropdown: t('Toggle Shelved Changes Dropdown'),
  ToggleDownloadCommitsDropdown: t('Toggle Download Commits Dropdown'),
  ToggleCwdDropdown: t('Toggle CWD Dropdown'),
  ToggleBulkActionsDropdown: t('Toggle Bulk Actions Dropdown'),
  ToggleFocusMode: t('Toggle Focus Mode'),
  ToggleBookmarksManagerDropdown: t('Toggle Bookmarks Manager Dropdown'),
  RebaseOntoCurrentStackBase: t('Rebase Selected Commits onto Current Stack Base'),
};

export function useShowKeyboardShortcutsHelp(): () => unknown {
  const showModal = useModal();
  const showShortcutsModal = () => {
    showModal({
      type: 'custom',
      component: () => (
        <div className="keyboard-shortcuts-menu">
          <table>
            {(Object.entries(ISLShortcutLabels) as Array<[ISLCommandName, string]>).map(
              ([name, label]) => {
                const [modifiers, keyCode] = allCommands[name];
                {
                  return (
                    <tr key={name}>
                      <td>{label}</td>
                      <td>
                        <Kbd
                          modifiers={Array.isArray(modifiers) ? modifiers : [modifiers]}
                          keycode={keyCode}
                        />
                      </td>
                    </tr>
                  );
                }
              },
            )}
          </table>
        </div>
      ),
      icon: 'keyboard',
      title: t('Keyboard Shortcuts'),
    });
  };
  useCommand('OpenShortcutHelp', showShortcutsModal);
  return showShortcutsModal;
}
