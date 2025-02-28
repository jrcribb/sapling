/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

#include "eden/fs/rust/redirect_ffi/include/ffi.h"

namespace facebook::eden {
Redirection redirectionFromFFI(RedirectionFFI&& redirFFI) {
  Redirection redir;
  redir.repoPath_ref() = std::string(std::move(redirFFI.repo_path));
  redir.redirType_ref() = redirFFI.redir_type;
  redir.source_ref() = std::string(std::move(redirFFI.source));
  redir.state_ref() = redirFFI.state;
  auto optTarget = redirectionTargetFromFFI(std::move(redirFFI.target));
  if (optTarget.has_value()) {
    redir.target_ref() = std::move(optTarget.value());
  }
  return redir;
}

std::optional<std::string> redirectionTargetFromFFI(
    rust::String&& redirTargetFFI) {
  if (redirTargetFFI.empty()) {
    return std::nullopt;
  }
  return std::string(std::move(redirTargetFFI));
}

} // namespace facebook::eden
