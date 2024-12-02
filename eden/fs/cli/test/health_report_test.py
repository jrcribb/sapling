#!/usr/bin/env python3
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This software may be used and distributed according to the terms of the
# GNU General Public License version 2.

# pyre-strict

import argparse
import json
import os

import tempfile
import typing
import unittest
from datetime import datetime
from unittest.mock import MagicMock, patch

from eden.fs.cli.version import VersionInfo

from eden.test_support.temporary_directory import TemporaryDirectoryMixin

from ..main import HealthReportCmd

# The version "20241030-165642" is the latest version.
# The version "20240928-144752" is over 30 days old, which is considered stale.
# The version "20241010-189752" is less than 30 days old, which is considered acceptable.
LATEST_VERSION_STR = "20241030-165642"
STALE_VERSION_STR = "20240928-144752"
ACCEPTABLE_VERSION_STR = "20241010-189752"


def get_version_age(version_str: str) -> int:
    return max((datetime.now() - datetime.strptime(version_str[:8], "%Y%m%d")).days, 0)


latest_version_age: int = get_version_age(LATEST_VERSION_STR)
stale_version_age: int = get_version_age(STALE_VERSION_STR)
acceptable_version_age: int = get_version_age(ACCEPTABLE_VERSION_STR)

latest_version: typing.Tuple[str] = (LATEST_VERSION_STR,)
stale_version: typing.Tuple[str] = (STALE_VERSION_STR,)
acceptable_version: typing.Tuple[str] = (ACCEPTABLE_VERSION_STR,)

latest_running_version_info = VersionInfo(
    LATEST_VERSION_STR,  # running version string
    latest_version_age,  # running version age
    LATEST_VERSION_STR,  # installed version string
    latest_version_age,  # installed version age
    0,  # diff between running and installed version
    True,  # is eden running
    True,  # is dev version
)
stale_running_version_info = VersionInfo(
    STALE_VERSION_STR,  # running version string
    stale_version_age,  # running version age
    LATEST_VERSION_STR,  # installed version string
    latest_version_age,  # installed version age
    stale_version_age
    - latest_version_age,  # diff between running and installed version
    True,  # is eden running
    True,  # is dev version
)
acceptable_running_version_info = VersionInfo(
    ACCEPTABLE_VERSION_STR,  # running version string
    acceptable_version_age,  # running version age
    LATEST_VERSION_STR,  # installed version string
    latest_version_age,  # installed version age
    acceptable_version_age
    - latest_version_age,  # diff between running and installed version
    True,  # is eden running
    True,  # is dev version
)


class HealthReportTest(unittest.TestCase, TemporaryDirectoryMixin):
    def setup(self) -> typing.Tuple[MagicMock, argparse.Namespace, str]:
        temp_dir = self.make_temporary_directory()
        eden_path = os.path.join(temp_dir, "mount_dir")

        args = argparse.Namespace(
            config_dir="/home/johndoe/.eden",
            etc_eden_dir="/etc/eden",
            home_dir="/home/johndoe",
            mount=eden_path,
            only_repo_source=True,
            mounts=[
                "/data/users/vinigupta/configerator_test",
                "/data/users/vinigupta/fbsource_test",
                "/data/users/vinigupta/opsfiles_test",
            ],
        )
        mock_argument_parser = MagicMock(spec=argparse.ArgumentParser)

        # Define the JSON data
        data = {
            "chef.run_success_timestamp": 1732557634,
            "chef.last_failure_time": 1732305759,
            "chef.last_success_time": 1732557634,
            "chef.run_success": True,
            "chef.run_timestamp": 1732557634,
            "chef.consecutive_failures": 0,
            "chef.run_start_time": 1732547468,
            "chef.run_end_time": 1732557624,
            "chef.run_elapsed_time": 10155,
            "chef.run_updated_resources_count": 27,
        }

        # Create a temporary file
        fd, file_path = tempfile.mkstemp()
        try:
            # Open the file in write mode
            with os.fdopen(fd, "w") as tmp_file:
                # Write the JSON data to the file
                json.dump(data, tmp_file)
        except Exception as e:
            print(f"An error occurred: {e}")

        return (mock_argument_parser, args, file_path)

    @patch("eden.fs.cli.util.get_chef_log_path")
    @patch("eden.fs.cli.doctor.facebook.check_x509.find_x509_path")
    @patch("eden.fs.cli.doctor.facebook.check_x509.validate_x509")
    @patch("eden.fs.cli.config.EdenInstance.get_running_version")
    @patch("eden.fs.cli.version.get_version_info")
    @patch("eden.fs.cli.util.HealthStatus.is_healthy")
    def test_calling_into_health_report(
        self,
        mock_is_healthy: MagicMock,
        mock_get_version_info: MagicMock,
        mock_get_running_version: MagicMock,
        mock_validate_x509: MagicMock,
        mock_find_x509_path: MagicMock,
        mock_get_chef_log_path: MagicMock,
    ) -> None:
        mock_argument_parser, args, file_path = self.setup()
        mock_get_chef_log_path.return_value = file_path
        mock_get_running_version.return_value = latest_version
        mock_get_version_info.return_value = latest_running_version_info
        mock_is_healthy.return_value = True
        mock_find_x509_path.return_value = ("some_cert_path",)
        mock_validate_x509.return_value = True

        test_health_report_cmd = HealthReportCmd(mock_argument_parser)
        result = test_health_report_cmd.run(args)
        self.assertIsNotNone(result)

    @patch("eden.fs.cli.util.HealthStatus.is_healthy")
    def test_health_report_notify_eden_not_running(
        self,
        mock_is_healthy: MagicMock,
    ) -> None:
        mock_argument_parser, args, file_path = self.setup()
        mock_is_healthy.return_value = False

        test_health_report_cmd = HealthReportCmd(mock_argument_parser)
        result = test_health_report_cmd.run(args)
        self.assertEqual(
            HealthReportCmd.error_codes,
            {
                HealthReportCmd.ErrorCode.EDEN_NOT_RUNNING: "Could not find EdenFS daemon pid.",
            },
        )

        self.assertEqual(result, 1)

    @patch("eden.fs.cli.util.get_chef_log_path")
    @patch("eden.fs.cli.doctor.facebook.check_x509.find_x509_path")
    @patch("eden.fs.cli.doctor.facebook.check_x509.validate_x509")
    @patch("eden.fs.cli.config.EdenInstance.get_running_version")
    @patch("eden.fs.cli.version.get_version_info")
    @patch("eden.fs.cli.util.HealthStatus.is_healthy")
    def test_health_report_check_for_stale_eden_version_prompt_error(
        self,
        mock_is_healthy: MagicMock,
        mock_get_version_info: MagicMock,
        mock_get_running_version: MagicMock,
        mock_validate_x509: MagicMock,
        mock_find_x509_path: MagicMock,
        mock_get_chef_log_path: MagicMock,
    ) -> None:
        mock_argument_parser, args, file_path = self.setup()
        mock_get_chef_log_path.return_value = file_path
        mock_get_running_version.return_value = stale_version
        mock_get_version_info.return_value = stale_running_version_info
        mock_is_healthy.return_value = True
        mock_find_x509_path.return_value = ("some_cert_path",)
        mock_validate_x509.return_value = True

        test_health_report_cmd = HealthReportCmd(mock_argument_parser)
        result = test_health_report_cmd.run(args)
        self.assertEqual(
            HealthReportCmd.error_codes,
            {
                HealthReportCmd.ErrorCode.STALE_EDEN_VERSION: "Running EdenFS version: 20240928-144752, installed EdenFS version: 20241030-165642"
            },
        )
        self.assertEqual(result, 1)

    @patch("eden.fs.cli.util.get_chef_log_path")
    @patch("eden.fs.cli.doctor.facebook.check_x509.find_x509_path")
    @patch("eden.fs.cli.doctor.facebook.check_x509.validate_x509")
    @patch("eden.fs.cli.config.EdenInstance.get_running_version")
    @patch("eden.fs.cli.version.get_version_info")
    @patch("eden.fs.cli.util.HealthStatus.is_healthy")
    def test_health_report_check_for_stale_eden_version_no_error(
        self,
        mock_is_healthy: MagicMock,
        mock_get_version_info: MagicMock,
        mock_get_running_version: MagicMock,
        mock_validate_x509: MagicMock,
        mock_find_x509_path: MagicMock,
        mock_get_chef_log_path: MagicMock,
    ) -> None:
        mock_argument_parser, args, file_path = self.setup()
        mock_get_chef_log_path.return_value = file_path
        mock_get_running_version.return_value = acceptable_version
        mock_get_version_info.return_value = acceptable_running_version_info
        mock_is_healthy.return_value = True
        mock_find_x509_path.return_value = ("some_cert_path",)
        mock_validate_x509.return_value = True

        test_health_report_cmd = HealthReportCmd(mock_argument_parser)
        result = test_health_report_cmd.run(args)
        self.assertIsNotNone(result)

    @patch("eden.fs.cli.util.get_chef_log_path")
    @patch("eden.fs.cli.doctor.facebook.check_x509.find_x509_path")
    @patch("eden.fs.cli.doctor.facebook.check_x509.validate_x509")
    @patch("eden.fs.cli.config.EdenInstance.get_running_version")
    @patch("eden.fs.cli.version.get_version_info")
    @patch("eden.fs.cli.util.HealthStatus.is_healthy")
    def test_health_report_check_for_invalid_certs(
        self,
        mock_is_healthy: MagicMock,
        mock_get_version_info: MagicMock,
        mock_get_running_version: MagicMock,
        mock_validate_x509: MagicMock,
        mock_find_x509_path: MagicMock,
        mock_get_chef_log_path: MagicMock,
    ) -> None:
        mock_argument_parser, args, file_path = self.setup()
        mock_get_chef_log_path.return_value = file_path
        mock_find_x509_path.return_value = ("some_cert_path",)
        mock_validate_x509.return_value = False
        mock_get_running_version.return_value = acceptable_version
        mock_get_version_info.return_value = acceptable_running_version_info
        mock_is_healthy.return_value = True

        test_health_report_cmd = HealthReportCmd(mock_argument_parser)
        result = test_health_report_cmd.run(args)
        self.assertEqual(
            HealthReportCmd.error_codes,
            {
                HealthReportCmd.ErrorCode.INVALID_CERTS: "Couldn't validate x509 certificates."
            },
        )

        self.assertEqual(result, 1)
