-- Number of measurements over time and insertion rate per bucket.
-- Change the interval in date_bin(...) to adjust the bucket size.
WITH params AS (
  SELECT INTERVAL '1 hour' AS bucket_width
),
buckets AS (
  SELECT
    date_bin(params.bucket_width, measurements.ts, TIMESTAMPTZ '1970-01-01') AS bucket_start,
    count(*) AS inserted_measurements
  FROM measurements
  CROSS JOIN params
  GROUP BY bucket_start
)
SELECT
  buckets.bucket_start,
  buckets.inserted_measurements,
  sum(buckets.inserted_measurements) OVER (ORDER BY buckets.bucket_start) AS total_inserted_measurements,
  buckets.inserted_measurements::double precision / extract(epoch FROM params.bucket_width) AS measurements_per_second,
  buckets.inserted_measurements::double precision * 60 / extract(epoch FROM params.bucket_width) AS measurements_per_minute,
  buckets.inserted_measurements::double precision * 3600 / extract(epoch FROM params.bucket_width) AS measurements_per_hour
FROM buckets
CROSS JOIN params
ORDER BY buckets.bucket_start;
