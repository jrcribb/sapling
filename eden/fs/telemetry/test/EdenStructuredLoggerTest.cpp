/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

#include "eden/fs/telemetry/EdenStructuredLogger.h"

#include <folly/json/json.h>
#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include "eden/common/telemetry/LogEvent.h"
#include "eden/common/telemetry/ScribeLogger.h"

using namespace facebook::eden;
using namespace testing;

namespace {

struct TestScribeLogger : public ScribeLogger {
  std::vector<std::string> lines;

  void log(std::string line) override {
    lines.emplace_back(std::move(line));
  }
};

struct TestLogEvent : public TestEvent {
  std::string str;
  int number = 0;

  TestLogEvent(std::string str, int number)
      : str(std::move(str)), number(number) {}

  void populate(DynamicEvent& event) const override {
    event.addString("str", str);
    event.addInt("number", number);
  }

  char const* getType() const override {
    return "test_event";
  }
};

struct TypelessTestLogEvent : public TypelessTestEvent {
  std::string str;
  int number = 0;

  TypelessTestLogEvent(std::string str, int number)
      : str(std::move(str)), number(number) {}

  void populate(DynamicEvent& event) const override {
    event.addString("str", str);
    event.addInt("number", number);
  }
};

struct EdenStructuredLoggerTest : public ::testing::Test {
  std::shared_ptr<TestScribeLogger> scribe{
      std::make_shared<TestScribeLogger>()};
  EdenStructuredLogger logger{
      scribe,
      SessionInfo{},
  };
};

} // namespace

std::vector<std::string> keysOf(const folly::dynamic& d) {
  std::vector<std::string> rv;
  for (const auto& key : d.keys()) {
    rv.push_back(key.asString());
  }
  return rv;
}

TEST_F(EdenStructuredLoggerTest, json_contains_types_at_top_level_and_values) {
  logger.logEvent(TestLogEvent{"name", 10});
  EXPECT_EQ(1, scribe->lines.size());
  const auto& line = scribe->lines[0];
  auto doc = folly::parseJson(line);
  EXPECT_TRUE(doc.isObject());
  EXPECT_THAT(keysOf(doc), UnorderedElementsAre("int", "normal"));

  auto ints = doc["int"];
  EXPECT_TRUE(ints.isObject());
  EXPECT_THAT(
      keysOf(ints), UnorderedElementsAre("time", "number", "session_id"));

  auto normals = doc["normal"];
  EXPECT_TRUE(normals.isObject());
#if defined(__APPLE__)
  EXPECT_THAT(
      keysOf(normals),
      UnorderedElementsAre(
          "str",
          "logged_by",
          "edenver",
          "host",
          "osver",
          "os",
          "user",
          "type",
          "system_architecture"));
#else
  EXPECT_THAT(
      keysOf(normals),
      UnorderedElementsAre(
          "str",
          "logged_by",
          "edenver",
          "host",
          "osver",
          "os",
          "user",
          "type"));
#endif
}

TEST_F(
    EdenStructuredLoggerTest,
    typeless_json_doesnt_contain_type_at_top_level) {
  logger.logEvent(TypelessTestLogEvent{"another name", 12});
  EXPECT_EQ(1, scribe->lines.size());
  const auto& line = scribe->lines[0];
  auto doc = folly::parseJson(line);
  EXPECT_TRUE(doc.isObject());
  EXPECT_THAT(keysOf(doc), UnorderedElementsAre("int", "normal"));

  auto ints = doc["int"];
  EXPECT_TRUE(ints.isObject());
  EXPECT_THAT(
      keysOf(ints), UnorderedElementsAre("time", "number", "session_id"));

  auto normals = doc["normal"];
  EXPECT_TRUE(normals.isObject());
#if defined(__APPLE__)
  EXPECT_THAT(
      keysOf(normals),
      UnorderedElementsAre(
          "str",
          "logged_by",
          "edenver",
          "host",
          "osver",
          "os",
          "user",
          "system_architecture"));
#else
  EXPECT_THAT(
      keysOf(normals),
      UnorderedElementsAre(
          "str", "logged_by", "edenver", "host", "osver", "os", "user"));
#endif
}
