-- givn:begin:timeline
local ok, reporter = pcall(require, "givn-timeline")
if ok and type(reporter) == "table" and reporter.setup then
  reporter.setup()
end
-- givn:end:timeline

-- givn:begin:attribution
local attribution_ok, attribution = pcall(require, "givn-attribution")
if attribution_ok and type(attribution) == "table" and attribution.setup then
  attribution.setup()
end
-- givn:end:attribution
